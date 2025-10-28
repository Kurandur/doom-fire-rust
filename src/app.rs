use pixels::{Pixels, PixelsBuilder, SurfaceTexture};
use std::sync::Arc;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};
use winit_input_helper::WinitInputHelper;

use crate::DoomFire;

pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 240;
pub const FPS: u64 = 60;
pub const TIME_PER_FRAME: u64 = 1000 / FPS;

pub struct App {
    pub window: Option<Arc<Window>>,
    pub input: WinitInputHelper,
    pub pixels: Option<Pixels<'static>>,
    pub doom_fire: Option<DoomFire>,
}

impl App {}

impl ApplicationHandler for App {
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                #[cfg(not(target_arch = "wasm32"))]
                let start_time = Instant::now();

                if let (Some(pixels), Some(doom_fire)) = (&mut self.pixels, &mut self.doom_fire) {
                    let frame = pixels.frame_mut();

                    for i in 0..(WIDTH * HEIGHT) {
                        let color =
                            doom_fire.get_color_from_palette(doom_fire.fire_pixels[i] as usize);
                        let idx = i * 4;
                        frame[idx..idx + 4].copy_from_slice(&color);
                    }

                    doom_fire.do_fire(); // move this *outside* the loop, usually
                }

                if let Err(err) = self.pixels.as_ref().unwrap().render() {
                    event_loop.exit();
                }
                self.window.as_ref().unwrap().request_redraw();
                #[cfg(not(target_arch = "wasm32"))]
                {
                    use std::time::Duration;

                    let end_time = Instant::now();
                    let render_time = end_time - start_time;
                    if render_time < Duration::from_millis(TIME_PER_FRAME) {
                        use std::thread;

                        let waste_time = Duration::from_millis(TIME_PER_FRAME) - render_time;
                        thread::sleep(waste_time);
                    }
                }
            }
            WindowEvent::Resized(size) => {
                if let Err(err) = self
                    .pixels
                    .as_mut()
                    .unwrap()
                    .resize_surface(size.width, size.height)
                {
                    event_loop.exit()
                }
            }
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                if let PhysicalKey::Code(KeyCode::Escape) = event.physical_key {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        // pass in events
        self.input.process_device_event(&event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.input.end_step();

        if self.input.key_released(KeyCode::KeyQ)
            || self.input.close_requested()
            || self.input.destroyed()
        {
            println!(
                "The application was requsted to close or the 'Q' key was pressed, quiting the application"
            );
            event_loop.exit();
            return;
        }

        if self.input.key_pressed(KeyCode::KeyW) {
            println!("The 'W' key (US layout) was pressed on the keyboard");
        }
    }

    fn new_events(&mut self, _: &ActiveEventLoop, _: StartCause) {
        self.input.step();
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();
        let window = Arc::new(window);
        self.window = Some(window.clone());
        self.doom_fire = Some(DoomFire::new());

        self.pixels = {
            let (window_width, window_height) = window.inner_size().into();
            let surface_texture = SurfaceTexture::new(window_width, window_height, window.clone());
            match Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture) {
                Ok(pixels) => {
                    window.request_redraw();
                    Some(pixels)
                }
                Err(err) => {
                    event_loop.exit();
                    None
                }
            }
        };
    }
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
