use winit::{event::WindowEvent, window::Window};

use crate::app::{RenderContext, WgpuContext};

pub mod egui;
pub mod map;

pub trait AppComponent {
    fn label(&self) -> Option<&'static str> {
        None
    }

    fn on_window_event(&mut self, window: &Window, event: &WindowEvent);
    fn on_render(
        &mut self,
        window: &Window,
        render_context: Option<&mut RenderContext>,
        wgpu_context: &WgpuContext,
    );
}
