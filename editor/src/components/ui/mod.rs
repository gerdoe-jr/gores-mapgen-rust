pub mod bottom_panel;
pub mod context;
pub mod float;
pub mod left_panel;

use std::{cell::RefCell, rc::Rc};

use context::UiContext;
use egui::Context;
use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::State;
use wgpu::StoreOp;
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

use crate::app::{RenderContext, WgpuContext};

use super::AppComponent;

pub struct UiComponent {
    state: State,
    renderer: Renderer,

    context: UiContext,
}

impl UiComponent {
    pub fn new(
        context: UiContext,
        window: &Window,
        wgpu_context: Rc<RefCell<WgpuContext>>,
    ) -> Self {
        let egui_context = Context::default();

        // speed up this lazy ui
        egui_context.style_mut(|style| style.animation_time /= 2.0);

        let state = egui_winit::State::new(
            egui_context,
            egui::viewport::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
        );
        let renderer = Renderer::new(
            &wgpu_context.borrow().device,
            wgpu_context.borrow().config.format,
            None,
            1,
        );

        Self {
            state,
            renderer,
            context,
        }
    }
}

impl AppComponent for UiComponent {
    fn label(&self) -> &'static str {
        "ui_component"
    }
    fn on_user_input(&mut self, window: &Window, event: &WindowEvent) -> bool {
        let _ = self.state.on_window_event(window, event);

        self.state.egui_ctx().is_pointer_over_area()
    }

    fn on_render(
        &mut self,
        window: &Window,
        render_context: Option<&mut RenderContext>,
        wgpu_context: &Rc<RefCell<WgpuContext>>,
    ) {
        let wgpu_context = wgpu_context.borrow();
        if let Some(render_context) = render_context {
            let screen_descriptor = ScreenDescriptor {
                size_in_pixels: [wgpu_context.config.width, wgpu_context.config.height],
                pixels_per_point: 1.0,
            };

            let command_encoder = render_context
                .command_encoders
                .get_mut(self.label())
                .unwrap();

            self.state
                .egui_ctx()
                .set_pixels_per_point(screen_descriptor.pixels_per_point);

            let raw_input = self.state.take_egui_input(window);

            self.state.egui_ctx().begin_frame(raw_input);
            (self.context.runner())(self.state.egui_ctx());
            let full_output = self.state.egui_ctx().end_frame();

            self.state
                .handle_platform_output(window, full_output.platform_output);

            let tris = self
                .state
                .egui_ctx()
                .tessellate(full_output.shapes, self.state.egui_ctx().pixels_per_point());

            for (id, image_delta) in &full_output.textures_delta.set {
                self.renderer.update_texture(
                    &wgpu_context.device,
                    &wgpu_context.queue,
                    *id,
                    image_delta,
                );
            }

            self.renderer.update_buffers(
                &wgpu_context.device,
                &wgpu_context.queue,
                command_encoder,
                &tris,
                &screen_descriptor,
            );

            {
                let mut render_pass =
                    command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &render_context.surface_view,
                            resolve_target: None,
                            ops: egui_wgpu::wgpu::Operations {
                                load: egui_wgpu::wgpu::LoadOp::Load,
                                store: StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        label: Some(self.label()),
                        occlusion_query_set: None,
                    });
                self.renderer
                    .render(&mut render_pass, &tris, &screen_descriptor);
            }

            for x in &full_output.textures_delta.free {
                self.renderer.free_texture(x)
            }
        }
    }

    fn on_resize(&mut self, _size: PhysicalSize<u32>) {}
}
