use super::context::RenderableUi;

pub struct FloatWindowUi {}

impl RenderableUi for FloatWindowUi {
    fn ui_with(&mut self, ctx: &egui::Context) {
        egui::Window::new("winit + egui + wgpu says hello!")
            .resizable(true)
            .vscroll(true)
            .default_open(false)
            .show(ctx, |ui| {
                ui.label("Label!");

                if ui.button("Button!").clicked() {
                    println!("boom!")
                }

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(format!("Pixels per point: {}", ctx.pixels_per_point()));
                });
            });
    }
}
