use std::path::PathBuf;

use egui::Context;
use egui_file_dialog::FileDialog;

use super::context::RenderableUi;

pub struct LeftPanelUi {
    pub file_dialog: FileDialog,
    pub current_map: Option<PathBuf>,
}

impl LeftPanelUi {
    pub fn new() -> Self {
        Self {
            file_dialog: FileDialog::new(),
            current_map: None,
        }
    }
}

impl RenderableUi for LeftPanelUi {
    fn ui_with(&mut self, ctx: &Context) {
        egui::panel::SidePanel::left("main_left_panel").show(ctx, |ui| {
            if ui.button("Select map").clicked() {
                self.file_dialog.select_file();
            }

            let map_name = if let Some(map_path) = &self.current_map {
                map_path.file_name().unwrap().to_str().unwrap()
            } else {
                "none"
            };

            ui.label(format!("Selected file: {}", map_name));

            // update selected file
            if let Some(path) = self.file_dialog.update(ctx).selected() {
                self.current_map = Some(path.to_path_buf());
            }
        });
    }
}
