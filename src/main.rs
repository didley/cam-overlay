mod application;
mod config;
mod window;

use application::CamOverlayApplication;

fn main() {
    gstreamer::init().expect("Failed to initialize GStreamer");

    let app = CamOverlayApplication::new();
    std::process::exit(app.run());
}
