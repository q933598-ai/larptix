(() => {
  "use strict";
  const ENHANCED = "larptrixMediaEnhanced";

  function formatTime(value) {
    if (!Number.isFinite(value) || value < 0) return "0:00";
    const total = Math.floor(value);
    return Math.floor(total / 60) + ":" + String(total % 60).padStart(2, "0");
  }

  function waveformSeed(text) {
    let seed = 0;
    for (let i = 0; i < text.length; i += 1) seed = (seed * 31 + text.charCodeAt(i)) >>> 0;
    return seed || 1;
  }

  function buildWaveform(seedText, count = 36) {
    const wrap = document.createElement("div");
    wrap.className = "larptrix-waveform";
    let seed = waveformSeed(seedText);
    for (let i = 0; i < count; i += 1) {
      seed = (seed * 1664525 + 1013904223) >>> 0;
      const bar = document.createElement("span");
      bar.style.height = (18 + (seed % 68)) + "%";
      wrap.append(bar);
    }
    return wrap;
  }

  function enhanceAudio(audio) {
    if (!audio || audio.classList.contains(ENHANCED)) return;
    audio.classList.add(ENHANCED);
    audio.controls = false;

    const card = document.createElement("div");
    card.className = "larptrix-media-card larptrix-audio-card";

    const play = document.createElement("button");
    play.type = "button";
    play.className = "larptrix-media-play";
    play.setAttribute("aria-label", "Play voice message");
    play.textContent = "▶";

    const main = document.createElement("div");
    main.className = "larptrix-media-main";

    const top = document.createElement("div");
    top.className = "larptrix-media-top";
    const title = document.createElement("strong");
    title.textContent = "Voice message";
    const time = document.createElement("span");
    time.className = "larptrix-media-time";
    time.textContent = "0:00 / 0:00";
    top.append(title, time);

    const waveform = buildWaveform(audio.src || audio.currentSrc || "");
    const progress = document.createElement("input");
    progress.type = "range";
    progress.className = "larptrix-media-progress";
    progress.min = "0";
    progress.max = "1000";
    progress.value = "0";
    progress.setAttribute("aria-label", "Voice message position");

    main.append(top, waveform, progress);
    card.append(play, main);
    audio.parentNode?.insertBefore(card, audio);
    card.append(audio);
    audio.tabIndex = -1;
    audio.style.display = "none";

    const update = () => {
      const duration = Number.isFinite(audio.duration) ? audio.duration : 0;
      const current = Number.isFinite(audio.currentTime) ? audio.currentTime : 0;
      time.textContent = formatTime(current) + " / " + formatTime(duration);
      progress.value = duration ? String(Math.round((current / duration) * 1000)) : "0";
      waveform.style.setProperty("--larptrix-progress", duration ? String(current / duration) : "0");
      play.textContent = audio.paused ? "▶" : "Ⅱ";
    };

    play.addEventListener("click", () => {
      if (audio.paused) void audio.play().catch(() => {});
      else audio.pause();
    });
    progress.addEventListener("input", () => {
      if (Number.isFinite(audio.duration) && audio.duration > 0) {
        audio.currentTime = (Number(progress.value) / 1000) * audio.duration;
      }
    });
    ["loadedmetadata", "timeupdate", "play", "pause", "ended"].forEach((event) => audio.addEventListener(event, update));
    update();
  }

  function enhanceVideo(video) {
    if (!video || video.classList.contains(ENHANCED)) return;
    video.classList.add(ENHANCED);
    video.controls = false;

    const card = document.createElement("div");
    card.className = "larptrix-video-card";
    const controls = document.createElement("div");
    controls.className = "larptrix-video-controls";

    const play = document.createElement("button");
    play.type = "button";
    play.className = "larptrix-video-play";
    play.setAttribute("aria-label", "Play video");
    play.textContent = "▶";

    const progress = document.createElement("input");
    progress.type = "range";
    progress.className = "larptrix-video-progress";
    progress.min = "0";
    progress.max = "1000";
    progress.value = "0";
    progress.setAttribute("aria-label", "Video position");

    const time = document.createElement("span");
    time.className = "larptrix-video-time";
    time.textContent = "0:00";

    const fullscreen = document.createElement("button");
    fullscreen.type = "button";
    fullscreen.className = "larptrix-video-fullscreen";
    fullscreen.setAttribute("aria-label", "Fullscreen");
    fullscreen.textContent = "⛶";

    controls.append(play, progress, time, fullscreen);
    video.parentNode?.insertBefore(card, video);
    card.append(video, controls);

    const update = () => {
      const duration = Number.isFinite(video.duration) ? video.duration : 0;
      const current = Number.isFinite(video.currentTime) ? video.currentTime : 0;
      progress.value = duration ? String(Math.round((current / duration) * 1000)) : "0";
      time.textContent = formatTime(current) + (duration ? " / " + formatTime(duration) : "");
      play.textContent = video.paused ? "▶" : "Ⅱ";
    };

    play.addEventListener("click", (event) => {
      event.stopPropagation();
      if (video.paused) void video.play().catch(() => {});
      else video.pause();
    });
    progress.addEventListener("input", (event) => {
      event.stopPropagation();
      if (Number.isFinite(video.duration) && video.duration > 0) {
        video.currentTime = (Number(progress.value) / 1000) * video.duration;
      }
    });
    fullscreen.addEventListener("click", async (event) => {
      event.stopPropagation();
      try {
        if (document.fullscreenElement) await document.exitFullscreen();
        else await video.requestFullscreen();
      } catch {}
    });
    video.addEventListener("click", (event) => {
      if (event.target.closest(".larptrix-video-controls")) return;
      if (video.paused) void video.play().catch(() => {});
      else video.pause();
    });
    ["loadedmetadata", "timeupdate", "play", "pause", "ended"].forEach((event) => video.addEventListener(event, update));
    update();
  }

  function enhance(root = document) {
    root.querySelectorAll("audio:not(." + ENHANCED + ")").forEach(enhanceAudio);
    root.querySelectorAll("video.chat-video:not(." + ENHANCED + ")").forEach(enhanceVideo);
  }

  enhance();
  new MutationObserver((mutations) => {
    for (const mutation of mutations) {
      for (const node of mutation.addedNodes) {
        if (node.nodeType === Node.ELEMENT_NODE) enhance(node);
      }
    }
  }).observe(document.body, { childList: true, subtree: true });
})();