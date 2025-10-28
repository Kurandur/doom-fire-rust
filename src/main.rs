use std::sync::{Arc, Mutex};

use doom_fire_rust::app::{App, Inner};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::dpi::LogicalSize;
use winit_input_helper::WinitInputHelper;

pub const FPS: u64 = 60;
pub const TIME_PER_FRAME: u64 = 1000 / FPS;

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Trace).expect("error initializing logger");

        wasm_bindgen_futures::spawn_local(run());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();

        pollster::block_on(run());
    }
}

async fn run() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop
        .run_app(&mut App {
            input: WinitInputHelper::new(),
            window: None,
            inner: Arc::new(Mutex::new(Inner {
                pixels: Mutex::new(None),
                doom_fire: Mutex::new(None),
            })),
        })
        .expect("Failed to run app");
}

#[cfg(target_arch = "wasm32")]
/// Retrieve current width and height dimensions of browser client window
fn get_window_size() -> LogicalSize<f64> {
    let client_window = web_sys::window().unwrap();
    LogicalSize::new(
        client_window.inner_width().unwrap().as_f64().unwrap(),
        client_window.inner_height().unwrap().as_f64().unwrap(),
    )
}
