use std::time::Instant;

use dear_imgui_rs::Context;
use dear_imgui_wgpu::WgpuRenderer;
use dear_imgui_winit::WinitPlatform;

pub struct ImguiState {
    pub context: Context,
    pub platform: WinitPlatform,
    pub renderer: WgpuRenderer,
    pub last_frame: Instant,
}
