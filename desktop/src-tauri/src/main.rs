use std::path::Path;
use std::process::Command;
use tauri::{WebviewUrl, WebviewWindowBuilder};
use url::Url;

#[tauri::command]
fn open_call_in_browser(window: tauri::WebviewWindow, url: String) -> Result<(), String> {
    let target = Url::parse(&url).map_err(|_| "invalid browser URL".to_string())?;
    if !matches!(target.scheme(), "http" | "https") || target.host_str().is_none() {
        return Err("only HTTP(S) URLs can be opened".to_string());
    }
    let server = std::env::var("LARPTRIX_SERVER_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
    let server = Url::parse(&server).map_err(|_| "invalid Larptrix server URL".to_string())?;
    let current = window
        .url()
        .map_err(|_| "could not read desktop webview URL".to_string())?;
    if current.origin() != server.origin() || target.origin() != server.origin() {
        return Err("browser URL must use the configured Larptrix server".to_string());
    }
    Command::new("xdg-open")
        .arg(target.as_str())
        .spawn()
        .map(|_| ())
        .map_err(|err| format!("could not open system browser: {err}"))
}

fn main() {
    if std::env::var_os("WAYLAND_DISPLAY").is_some()
        && std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none()
    {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }
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
            let server = std::env::var("LARPTRIX_SERVER_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
            let server_url = Url::parse(&server)?;
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(server_url))
                .title("Larptrix")
                .inner_size(1200.0, 800.0)
                .min_inner_size(800.0, 560.0)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run Larptrix desktop client");
}
