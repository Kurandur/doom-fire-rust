use dear_imgui_rs::Context;
use dear_imgui_rs::*;
use dear_imgui_wgpu::WgpuRenderer;
use dear_imgui_winit::WinitPlatform;
use glow::HasContext;
use pixels::{Pixels, PixelsBuilder, SurfaceTexture};
use pollster::block_on;
use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::DoomFire;
use crate::imgui::ImguiState;
use dear_imgui_glow::GlowRenderer;
use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext},
    display::{GetGlDisplay, GlDisplay},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface},
};
use raw_window_handle::HasWindowHandle;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};
use winit_input_helper::WinitInputHelper;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 240;
pub const FPS: u64 = 60;
pub const TIME_PER_FRAME: u64 = 1000 / FPS;

pub struct App {
    pub state: Option<AppState>,
}

pub struct AppState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    window: Arc<Window>,
    surface_desc: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface<'static>,
    imgui: ImguiState,
    doom_fire: DoomFire,
    //pixels: Arc<Mutex<Pixels<'static>>>,
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

impl App {
    pub fn new() -> Self {
        App { state: None }
    }
}

impl AppState {
    pub fn new(event_loop: &ActiveEventLoop) -> Result<Self, Box<dyn std::error::Error>> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

        let window = {
            let size = LogicalSize::new(1280.0, 720.0);
            Arc::new(
                event_loop.create_window(
                    Window::default_attributes()
                        .with_title("Dear ImGui WGPU - Texture Demo")
                        .with_inner_size(size),
                )?,
            )
        };

        let surface = instance.create_surface(window.clone())?;
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("No suitable GPU adapters found on the system!");

        let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))?;

        let size = LogicalSize::new(1280.0, 720.0);
        let caps = surface.get_capabilities(&adapter);
        let preferred_srgb = [
            wgpu::TextureFormat::Bgra8UnormSrgb,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        ];
        let format = preferred_srgb
            .iter()
            .cloned()
            .find(|f| caps.formats.contains(f))
            .unwrap_or(caps.formats[0]);

        let surface_desc = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width as u32,
            height: size.height as u32,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_desc);

        // ImGui context
        let mut context = Context::create();
        context.set_ini_filename(None::<String>).unwrap();
        let mut platform = WinitPlatform::new(&mut context);
        platform.attach_window(&window, dear_imgui_winit::HiDpiMode::Default, &mut context);

        // Renderer
        let init_info =
            dear_imgui_wgpu::WgpuInitInfo::new(device.clone(), queue.clone(), surface_desc.format);
        let mut renderer = WgpuRenderer::new(init_info, &mut context)?;
        renderer.set_gamma_mode(dear_imgui_wgpu::GammaMode::Auto);

        let imgui = ImguiState {
            context,
            platform,
            renderer,
            last_frame: Instant::now(),
        };
        let doom_fire = DoomFire::new();
        let size = window.inner_size();
        let window_width = size.width;
        let window_height = size.height;
        //let surface_texture = SurfaceTexture::new(window_width, window_height, window.clone());
        //let pixels = Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?;
        Ok(Self {
            device,
            queue,
            window,
            surface_desc,
            surface,
            imgui,
            doom_fire,
            //pixels: Arc::new(Mutex::new(pixels)),
        })
    }
}

impl ApplicationHandler for App {
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let state = match self.state.as_mut() {
            Some(state) => state,
            None => return,
        };

        let full_event: winit::event::Event<()> = winit::event::Event::WindowEvent {
            window_id,
            event: event.clone(),
        };
        state
            .imgui
            .platform
            .handle_event(&mut state.imgui.context, &state.window, &full_event);

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let delta_time = now - state.imgui.last_frame;
                let delta_secs = delta_time.as_secs_f32();
                state.imgui.context.io_mut().set_delta_time(delta_secs);
                state.imgui.last_frame = now;
                let frame = state.surface.get_current_texture().unwrap();
                state
                    .imgui
                    .platform
                    .prepare_frame(&state.window, &mut state.imgui.context);
                let ui = state.imgui.context.frame();
                ui.window("Hello")
                    .size([360.0, 180.0], Condition::FirstUseEver)
                    .build(|| {
                        ui.text("Hello, world!");

                        ui.same_line();

                        ui.separator();
                        ui.text(format!("Frame {:.2} ms", ui.io().delta_time() * 1000.0));
                    });
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder =
                    state
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Render Encoder"),
                        });

                // Finalize inputs on platform and build draw data
                state
                    .imgui
                    .platform
                    .prepare_render_with_ui(&ui, &state.window);
                let draw_data = state.imgui.context.render();
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    state.imgui.renderer.new_frame().unwrap();
                    state
                        .imgui
                        .renderer
                        .render_draw_data(draw_data, &mut rpass)
                        .unwrap();
                }
                state.queue.submit(Some(encoder.finish()));
                frame.present();
                state.window.request_redraw();
            }
            WindowEvent::Resized(size) => {}
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {}
            _ => {}
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        //self.input.process_device_event(&event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        //self.input.end_step();
        //
        //if self.input.key_released(KeyCode::KeyQ)
        //    || self.input.close_requested()
        //    || self.input.destroyed()
        //{
        //    println!(
        //        "The application was requsted to close or the 'Q' key was pressed, quiting the application"
        //    );
        //    event_loop.exit();
        //    return;
        //}

        //if self.input.key_pressed(KeyCode::KeyW) {
        //    println!("The 'W' key (US layout) was pressed on the keyboard");
        //}
    }

    fn new_events(&mut self, _: &ActiveEventLoop, _: StartCause) {
        //self.input.step();
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            match AppState::new(event_loop) {
                Ok(app_state) => {
                    self.state = Some(app_state);
                }
                Err(e) => {
                    eprintln!("Failed to create app state: {e}");
                    event_loop.exit();
                }
            }
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
