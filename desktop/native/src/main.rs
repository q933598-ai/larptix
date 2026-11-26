mod crypto;
mod network;
mod shell;

use std::future::pending;
use std::thread;

use gpui::{px, size, App, AppContext, Bounds, WindowBounds, WindowOptions};
use gpui_platform::application;
use shell::LarptrixShell;

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("larptrix-network")
        .build()
        .expect("could not start the network runtime");
    let runtime_handle = runtime.handle().clone();

    thread::Builder::new()
        .name("larptrix-runtime".into())
        .spawn(move || runtime.block_on(pending::<()>()))
        .expect("could not start the network thread");

    application().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1280.0), px(820.0)), cx);
        let handle = runtime_handle.clone();

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |_, cx| cx.new(|cx| LarptrixShell::new(cx, handle.clone())),
        )
        .expect("could not open Larptrix desktop window");

        cx.activate(true);
    });
}
