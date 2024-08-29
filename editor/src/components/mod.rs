use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

use crate::app::{RenderContext, WgpuContext};

pub mod egui;
pub mod map;

pub trait AppComponent {
    fn label(&self) -> Option<&'static str> {
        None
    }

    fn on_user_input(&mut self, window: &Window, event: &WindowEvent);
    fn on_render(
        &mut self,
        window: &Window,
        render_context: Option<&mut RenderContext>,
        wgpu_context: &WgpuContext,
    );
    fn on_resize(&mut self, size: PhysicalSize<u32>);
}
