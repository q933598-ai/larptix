mod shell;

use gpui::{px, size, App, Application, Bounds, WindowBounds, WindowOptions};
use shell::LarptrixShell;

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1280.0), px(820.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| LarptrixShell::default()),
        )
        .expect("could not open Larptrix desktop window");

        cx.activate(true);
    });
}
