#[cfg(not(target_os = "android"))]
use std::path::Path;
#[cfg(not(target_os = "android"))]
use std::process::Command;

use tauri::{WebviewUrl, WebviewWindowBuilder};
use url::Url;

fn validate_http_url(raw: &str) -> Result<Url, String> {
    let url = Url::parse(raw.trim()).map_err(|_| "invalid Larptix URL".to_string())?;

    if !matches!(url.scheme(), "http" | "https") {
        return Err("only HTTP(S) URLs are allowed".to_string());
    }

    if url.host_str().is_none() {
        return Err("Larptix URL must contain a hostname".to_string());
    }

    if !url.username().is_empty() || url.password().is_some() {
        return Err("credentials in URLs are not allowed".to_string());
    }

    Ok(url)
}

#[tauri::command]
fn open_call_in_browser(window: tauri::WebviewWindow, url: String) -> Result<(), String> {
    let target = validate_http_url(&url)?;

    let current = window
        .url()
        .map_err(|_| "could not read current Larptix URL".to_string())?;

    if target.origin() != current.origin() {
        return Err("browser URL must use the currently connected Larptix server".to_string());
    }

    #[cfg(target_os = "android")]
    {
        let _ = target;
        return Err("opening an external browser is not available from the Android client".to_string());
    }

    #[cfg(not(target_os = "android"))]
    {
        Command::new("xdg-open")
            .arg(target.as_str())
            .spawn()
            .map(|_| ())
            .map_err(|err| format!("could not open system browser: {err}"))
    }
}

fn main() {
    #[cfg(not(target_os = "android"))]
    if std::env::var_os("WAYLAND_DISPLAY").is_some()
        && std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none()
    {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    #[cfg(not(target_os = "android"))]
    if std::env::var_os("APPIMAGE").is_some() && std::env::var_os("GST_PLUGIN_PATH_1_0").is_none() {
        let system_plugin_paths = [
            "/usr/lib/gstreamer-1.0",
            "/usr/lib64/gstreamer-1.0",
            "/usr/lib/x86_64-linux-gnu/gstreamer-1.0",
        ]
        .into_iter()
        .filter(|path| Path::new(path).is_dir())
        .collect::<Vec<_>>()
        .join(":");

        if !system_plugin_paths.is_empty() {
            std::env::set_var("GST_PLUGIN_PATH_1_0", system_plugin_paths);
        }
    }

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![open_call_in_browser])
        .setup(|app| {
            let url = match std::env::var("LARPTRIX_SERVER_URL") {
                Ok(server) => {
                    let server = validate_http_url(&server).map_err(std::io::Error::other)?;

                    WebviewUrl::External(server)
                }

                Err(_) => WebviewUrl::App("index.html".into()),
            };

            WebviewWindowBuilder::new(app, "main", url)
                .title("Larptix")
                .inner_size(1200.0, 800.0)
                .min_inner_size(800.0, 560.0)
                .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run Larptix desktop client");
}
