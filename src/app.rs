use pixels::{Pixels, PixelsBuilder, SurfaceTexture};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use winit::dpi::{LogicalSize, PhysicalSize};

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};
use winit_input_helper::WinitInputHelper;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

use crate::DoomFire;

pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 240;
pub const FPS: u64 = 60;
pub const TIME_PER_FRAME: u64 = 1000 / FPS;

pub struct Inner {
    pub pixels: Mutex<Option<Pixels<'static>>>,
    pub doom_fire: Mutex<Option<DoomFire>>,
}

pub struct App {
    pub window: Option<Arc<Window>>,
    pub input: WinitInputHelper,
    pub inner: Arc<Mutex<Inner>>,
}

#[cfg(target_arch = "wasm32")]
async fn init_pixels(window: Arc<Window>) -> Option<Pixels<'static>> {
    let logical_size = get_window_size();
    let window_size = logical_size.to_physical::<u32>(window.scale_factor());
    if window_size.width == 0 || window_size.height == 0 {
        return None;
    }
    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &*window);
    let texture_format = pixels::wgpu::TextureFormat::Rgba8Unorm;
    let builder = PixelsBuilder::new(WIDTH as u32, HEIGHT as u32, surface_texture)
        .texture_format(texture_format)
        .surface_texture_format(texture_format);
    match builder.build_async().await {
        Ok(pixels) => Some(unsafe { std::mem::transmute::<Pixels<'_>, Pixels<'static>>(pixels) }),
        Err(_) => None,
    }
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

                let inner_guard = self.inner.lock().unwrap();
                let mut pixels_guard = inner_guard.pixels.lock().unwrap();
                let mut doom_fire_guard = inner_guard.doom_fire.lock().unwrap();
                if pixels_guard.is_some() && doom_fire_guard.is_some() {
                    let pixels = pixels_guard.as_mut().unwrap();
                    let doom_fire = doom_fire_guard.as_mut().unwrap();
                    let frame = pixels.frame_mut();

                    for i in 0..(WIDTH * HEIGHT) {
                        let color =
                            doom_fire.get_color_from_palette(doom_fire.fire_pixels[i] as usize);
                        let idx = i * 4;
                        frame[idx..idx + 4].copy_from_slice(&color);
                    }
                    doom_fire.do_fire();
                }

                if let Some(pixels) = pixels_guard.as_ref() {
                    if let Err(err) = pixels.render() {
                        event_loop.exit();
                    }
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
                let inner_guard = self.inner.lock().unwrap();
                let mut pixels_guard = inner_guard.pixels.lock().unwrap();
                if let Some(pixels) = pixels_guard.as_mut() {
                    if let Err(err) = pixels.resize_surface(size.width, size.height) {
                        event_loop.exit()
                    }
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

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowExtWebSys;

            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.body())
                .and_then(|body| {
                    body.append_child(&web_sys::Element::from(window.canvas().unwrap()))
                        .ok()
                })
                .expect("couldn't append canvas to document body");

            let closure = wasm_bindgen::closure::Closure::wrap(Box::new({
                let window = window.clone();
                move |_e: web_sys::Event| {
                    let _ = window.request_inner_size(get_window_size());
                }
            }) as Box<dyn FnMut(_)>);
            web_sys::window()
                .unwrap()
                .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();

            let _ = window.request_inner_size(get_window_size());
        }

        *self.inner.lock().unwrap().doom_fire.lock().unwrap() = Some(DoomFire::new());

        let size = window.inner_size();
        let window_width = size.width;
        let window_height = size.height;
        #[cfg(not(target_arch = "wasm32"))]
        {
            let surface_texture = SurfaceTexture::new(window_width, window_height, window.clone());
            match Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture) {
                Ok(pixels) => {
                    *self.inner.lock().unwrap().pixels.lock().unwrap() = Some(pixels);
                    window.request_redraw();
                }
                Err(err) => {
                    event_loop.exit();
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            *self.inner.lock().unwrap().pixels.lock().unwrap() = None;
            let inner_arc = self.inner.clone();
            let window_clone = window.clone();
            let window_clone2 = window.clone();
            spawn_local(async move {
                if let Some(pixels) = init_pixels(window_clone).await {
                    *inner_arc.lock().unwrap().pixels.lock().unwrap() = Some(pixels);
                    window_clone2.request_redraw();
                }
            });
        }
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
