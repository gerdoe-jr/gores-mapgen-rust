use egui::Context;

use super::context::{RenderableUi, UiState};

pub struct LeftPanelUi;

impl RenderableUi for LeftPanelUi {
    fn ui_with(&mut self, ctx: &Context, state: &mut UiState) {
        egui::panel::SidePanel::left("main_left_panel").show(ctx, |ui| {
            if ui.button("Select map").clicked() {
                state.file_dialog.select_file();
            }

            let map_name = if let Some(map_path) = &state.current_map {
                map_path.file_name().unwrap().to_str().unwrap()
            } else {
                "none"
            };

            ui.label(format!("Selected file: {}", map_name));

            // update selected file
            if let Some(path) = state.file_dialog.update(ctx).selected() {
                state.current_map = Some(path.to_path_buf());
            }
        });
    }
}