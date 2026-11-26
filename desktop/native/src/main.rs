mod shell;

use gpui::{App, Application, WindowOptions};
use shell::LarptrixShell;

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|_| LarptrixShell::default())
        })
        .expect("could not open Larptrix desktop window");

        cx.activate(true);
    });
}
