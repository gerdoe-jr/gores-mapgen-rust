use std::{cell::RefCell, path::PathBuf, rc::Rc};

use egui::{popup_below_widget, Context, Id};
use egui_file_dialog::{DialogState, FileDialog};
use twmap::TwMap;

use crate::components::map::MapLoader;

use super::context::RenderableUi;

pub struct LeftPanelUi {
    file_dialog: FileDialog,
    current_map: Option<PathBuf>,

    map_loader: Rc<RefCell<MapLoader>>,
}

impl LeftPanelUi {
    pub fn new(map_loader: Rc<RefCell<MapLoader>>) -> Self {
        Self {
            file_dialog: FileDialog::new(),
            current_map: None,
            map_loader,
        }
    }
}

impl RenderableUi for LeftPanelUi {
    fn ui_with(&mut self, ctx: &Context) {
        egui::panel::SidePanel::left("main_left_panel")
            .resizable(true)
            .show(ctx, |ui| {
                let map_loaded = self.map_loader.borrow().is_loaded();
                let response = ui.button(if !map_loaded {
                    "Load map"
                } else {
                    "Unload map"
                });

                if response.clicked() {
                    if !map_loaded {
                        self.file_dialog.select_file();
                    } else {
                        self.map_loader.borrow_mut().unload();
                        self.current_map = None;
                    }
                }

                let popup_id = Id::new("select_map_popup");

                let map_name = if let Some(map_path) = &self.current_map {
                    map_path.file_name().unwrap().to_str().unwrap()
                } else {
                    "none"
                };

                ui.horizontal(|ui| {
                    ui.label("Loaded map:");
                    ui.monospace(map_name);
                });

                if self.file_dialog.state() == DialogState::Open {
                    if let Some(path) = self.file_dialog.update(ctx).selected() {
                        match TwMap::parse_path(path) {
                            Ok(mut tw_map) => {
                                tw_map.load().unwrap(); // TODO: handle error
                                self.map_loader.borrow_mut().load(tw_map);
                                self.current_map = Some(path.to_path_buf());
                            }
                            Err(err) => {
                                popup_below_widget(ui, popup_id, &response, |ui| {
                                    ui.label(format!(
                                        "Failed to open '{}': {:?}",
                                        path.to_string_lossy(),
                                        err
                                    ));
                                });
                            }
                        }
                    }
                }
            });
    }
}
