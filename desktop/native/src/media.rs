use std::{
    path::Path,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};

use gpui::RenderImage;
use image::{Frame, ImageBuffer, Rgba};
use smallvec::smallvec;

static GSTREAMER_INIT: OnceLock<Result<(), String>> = OnceLock::new();
const MAX_VIDEO_FRAME_BYTES: usize = 64 * 1024 * 1024;

pub struct MediaPlayback {
    pipeline: gstreamer::Element,
    video_sink: Option<gstreamer_app::AppSink>,
    active: AtomicBool,
    playing: AtomicBool,
    finished: AtomicBool,
    latest_frame: Mutex<Option<Arc<RenderImage>>>,
}

impl MediaPlayback {
    pub fn open(path: &Path, video: bool) -> Result<Arc<Self>, String> {
        GSTREAMER_INIT
            .get_or_init(|| gstreamer::init().map_err(|error| error.to_string()))
            .as_ref()
            .map_err(Clone::clone)?;

        let uri = url::Url::from_file_path(path)
            .map_err(|_| "Could not create a local media URL".to_string())?
            .to_string();

        let video_sink = if video {
            let sink = gstreamer::ElementFactory::make("appsink")
                .name("larptrix-video-sink")
                .build()
                .map_err(|error| format!("Could not create the video output: {error}"))?
                .downcast::<gstreamer_app::AppSink>()
                .map_err(|_| "GStreamer returned an invalid video output".to_string())?;

            let caps = gstreamer::Caps::builder("video/x-raw")
                .field("format", "BGRA")
                .build();
            sink.set_caps(Some(&caps));
            sink.set_max_buffers(2);
            sink.set_property("drop", true);
            sink.set_property("sync", true);
            Some(sink)
        } else {
            None
        };

        let pipeline = gstreamer::ElementFactory::make("playbin")
            .name("larptrix-player")
            .property("uri", &uri)
            .build()
            .map_err(|error| format!("Could not create the media pipeline: {error}"))?;

        if let Some(sink) = video_sink.as_ref() {
            pipeline.set_property("video-sink", sink);
        }
        pipeline.set_property("volume", 0.8_f64);

        pipeline
            .set_state(gstreamer::State::Playing)
            .map_err(|error| format!("Could not start media playback: {error}"))?;

        Ok(Arc::new(Self {
            pipeline,
            video_sink,
            active: AtomicBool::new(true),
            playing: AtomicBool::new(true),
            finished: AtomicBool::new(false),
            latest_frame: Mutex::new(None),
        }))
    }

    pub fn set_playing(&self, playing: bool) -> Result<(), String> {
        if !self.active.load(Ordering::Acquire) {
            return Err("The media player is closed".into());
        }

        if playing && self.finished.load(Ordering::Acquire) {
            self.seek_to(0)?;
        }

        self.pipeline
            .set_state(if playing {
                gstreamer::State::Playing
            } else {
                gstreamer::State::Paused
            })
            .map_err(|error| format!("Could not change playback state: {error}"))?;
        self.playing.store(playing, Ordering::Release);
        self.finished.store(false, Ordering::Release);
        Ok(())
    }

    pub fn set_volume(&self, volume: f32) {
        self.pipeline
            .set_property("volume", f64::from(volume.clamp(0.0, 1.0)));
    }

    pub fn seek_to(&self, position_ms: u64) -> Result<(), String> {
        if !self.active.load(Ordering::Acquire) {
            return Err("The media player is closed".into());
        }

        self.pipeline
            .seek_simple(
                gstreamer::SeekFlags::FLUSH | gstreamer::SeekFlags::KEY_UNIT,
                gstreamer::ClockTime::from_mseconds(position_ms),
            )
            .map_err(|error| format!("This media stream cannot seek: {error}"))?;
        self.finished.store(false, Ordering::Release);
        Ok(())
    }

    pub fn poll_video_frame(&self) -> Result<(), String> {
        if let Some(frame) = self.take_video_frame()? {
            if let Ok(mut latest) = self.latest_frame.lock() {
                *latest = Some(frame);
            }
        }
        Ok(())
    }

    pub fn latest_video_frame(&self) -> Option<Arc<RenderImage>> {
        self.latest_frame.lock().ok().and_then(|frame| frame.clone())
    }

    pub fn position_ms(&self) -> u64 {
        self.pipeline
            .query_position::<gstreamer::ClockTime>()
            .map(|time| time.mseconds())
            .unwrap_or_default()
    }

    pub fn duration_ms(&self) -> u64 {
        self.pipeline
            .query_duration::<gstreamer::ClockTime>()
            .map(|time| time.mseconds())
            .unwrap_or_default()
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    pub fn is_playing(&self) -> bool {
        self.playing.load(Ordering::Acquire)
    }

    pub fn take_video_frame(&self) -> Result<Option<Arc<RenderImage>>, String> {
        let Some(sink) = self.video_sink.as_ref() else {
            return Ok(None);
        };

        let Some(sample) = sink.try_pull_sample(gstreamer::ClockTime::from_mseconds(2)) else {
            return Ok(None);
        };
        let caps = sample
            .caps()
            .ok_or_else(|| "The video decoder returned a frame without format information".to_string())?;
        let info = gstreamer_video::VideoInfo::from_caps(caps)
            .map_err(|error| format!("Could not read the video frame format: {error}"))?;
        let buffer = sample
            .buffer()
            .ok_or_else(|| "The video decoder returned an empty frame".to_string())?;
        let frame = gstreamer_video::VideoFrameRef::from_buffer_ref_readable(buffer, &info)
            .map_err(|error| format!("Could not access the decoded video frame: {error}"))?;

        let width = frame.width();
        let height = frame.height();
        let row_bytes = (width as usize)
            .checked_mul(4)
            .ok_or_else(|| "The video frame dimensions are too large".to_string())?;
        let total_bytes = row_bytes
            .checked_mul(height as usize)
            .filter(|size| *size <= MAX_VIDEO_FRAME_BYTES)
            .ok_or_else(|| "The video frame exceeds the 64 MiB display limit".to_string())?;
        let stride = frame.plane_stride()[0];
        if stride <= 0 || (stride as usize) < row_bytes {
            return Err("The video decoder returned an unsupported frame layout".into());
        }

        let source = frame
            .plane_data(0)
            .map_err(|error| format!("Could not read the decoded video pixels: {error}"))?;
        let stride = stride as usize;
        let mut pixels = vec![0_u8; total_bytes];
        for row in 0..height as usize {
            let source_start = row
                .checked_mul(stride)
                .ok_or_else(|| "The video frame layout overflowed".to_string())?;
            let source_end = source_start + row_bytes;
            let target_start = row * row_bytes;
            let target_end = target_start + row_bytes;
            let source_row = source
                .get(source_start..source_end)
                .ok_or_else(|| "The video decoder returned a truncated frame".to_string())?;
            pixels[target_start..target_end].copy_from_slice(source_row);
        }

        // GPUI's RenderImage stores pixels in BGRA order, matching the requested GStreamer caps.
        let image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width, height, pixels)
            .ok_or_else(|| "Could not construct the GPUI video frame".to_string())?;
        Ok(Some(Arc::new(RenderImage::new(smallvec![Frame::new(image)]))))
    }

    pub fn poll_finished(&self) -> bool {
        if self.finished.load(Ordering::Acquire) {
            return true;
        }

        let Some(bus) = self.pipeline.bus() else {
            return false;
        };
        if bus
            .pop_filtered(&[gstreamer::MessageType::Eos])
            .is_some()
        {
            self.playing.store(false, Ordering::Release);
            self.finished.store(true, Ordering::Release);
            return true;
        }
        false
    }

    pub fn stop(&self) {
        if self.active.swap(false, Ordering::AcqRel) {
            let _ = self.pipeline.set_state(gstreamer::State::Null);
            self.playing.store(false, Ordering::Release);
            if let Ok(mut frame) = self.latest_frame.lock() {
                *frame = None;
            }
        }
    }
}

impl Drop for MediaPlayback {
    fn drop(&mut self) {
        self.stop();
    }
}
