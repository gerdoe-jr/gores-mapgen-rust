use egui::{Context, RawInput};
use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::State;
use wgpu::{CommandEncoder, StoreOp, TextureView};
use winit::{event::WindowEvent, window::Window};

use crate::app::{RenderContext, WgpuContext};

use super::AppComponent;

pub struct EguiComponent {
    state: State,
    renderer: Renderer,
}

impl EguiComponent {
    pub fn new(window: &Window, wgpu_context: &WgpuContext) -> Self {
        let egui_context = Context::default();

        let state = egui_winit::State::new(
            egui_context,
            egui::viewport::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
        );
        let renderer = Renderer::new(&wgpu_context.device, wgpu_context.config.format, None, 1);

        Self { state, renderer }
    }

    pub fn draw(
        &mut self,
        wgpu_context: &WgpuContext,
        encoder: &mut CommandEncoder,
        window: &Window,
        window_surface_view: &TextureView,
        screen_descriptor: ScreenDescriptor,
        run_ui: impl FnOnce(&Context),
    ) {
        self.state
            .egui_ctx()
            .set_pixels_per_point(screen_descriptor.pixels_per_point);

        let raw_input = self.state.take_egui_input(window);

        let full_output = self.state.egui_ctx().run(raw_input, |ui| {
            run_ui(ui);
        });

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
            encoder,
            &tris,
            &screen_descriptor,
        );

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: window_surface_view,
                    resolve_target: None,
                    ops: egui_wgpu::wgpu::Operations {
                        load: egui_wgpu::wgpu::LoadOp::Load,
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                label: self.label(),
                occlusion_query_set: None,
            });
            self.renderer.render(&mut rpass, &tris, &screen_descriptor);
        }

        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x)
        }
    }
}

impl AppComponent for EguiComponent {
    fn label(&self) -> Option<&'static str> {
        Some("egui_component")
    }
    fn on_window_event(&mut self, window: &Window, event: &WindowEvent) {
        let _ = self.state.on_window_event(window, event);
    }

    fn on_render(
        &mut self,
        window: &Window,
        render_context: Option<&mut RenderContext>,
        wgpu_context: &WgpuContext,
    ) {
        if let Some(render_context) = render_context {
            let screen_descriptor = ScreenDescriptor {
                size_in_pixels: [wgpu_context.config.width, wgpu_context.config.height],
                pixels_per_point: 1.0,
            };

            self.draw(
                wgpu_context,
                &mut render_context
                    .command_encoders
                    .get_mut(self.label().unwrap())
                    .unwrap(),
                &window,
                &render_context.surface_view,
                screen_descriptor,
                |ctx| {
                    egui::Window::new("winit + egui + wgpu + twgpu says hello!")
                        .resizable(true)
                        .vscroll(true)
                        .default_open(false)
                        .show(ctx, |ui| {
                            ui.label("Label!");

                            if ui.button("Button!").clicked() {
                                println!("button!")
                            }
                        });
                },
            );
        }
    }
}
