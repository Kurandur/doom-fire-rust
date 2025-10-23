use std::rc::Rc;

use pixels::{PixelsBuilder, SurfaceTexture};
use winit::event::Event;
use winit::keyboard::KeyCode;
use winit::{dpi::LogicalSize, event::WindowEvent, event_loop::EventLoop, window::WindowBuilder};
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;

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
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Doom Fire")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .expect("WindowBuilder error")
    };
    let window = Rc::new(window);
    let mut input = WinitInputHelper::new();
    let mut pixels = {
        #[cfg(not(target_arch = "wasm32"))]
        let window_size = window.inner_size();

        #[cfg(target_arch = "wasm32")]
        let window_size = get_window_size().to_physical::<u32>(window.scale_factor());

        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, window.as_ref());
        let builder = PixelsBuilder::new(WIDTH, HEIGHT, surface_texture);

        #[cfg(target_arch = "wasm32")]
        let builder = {
            // Web targets do not support the default texture format
            let texture_format = pixels::wgpu::TextureFormat::Rgba8Unorm;
            builder
                .texture_format(texture_format)
                .surface_texture_format(texture_format)
        };

        builder.build_async().await.expect("Pixels error")
    };

    let res = event_loop.run(|event, elwt| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                if let Err(err) = pixels.render() {
                    //log_error("pixels.render", err);
                    elwt.exit();
                    return;
                }

                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {}
            _ => (),
        }
        if input.update(&event) && (input.key_pressed(KeyCode::Escape) || input.close_requested()) {
            elwt.exit();
        }
    });
    res.unwrap();
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
