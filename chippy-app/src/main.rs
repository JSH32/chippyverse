#![feature(slice_flatten)]

use app::MainApp;

use window::WindowContainer;

mod app;
mod debugger;
mod input;
mod window;

fn main() {
    WindowContainer::new(MainApp::new()).open().join().unwrap();
}
