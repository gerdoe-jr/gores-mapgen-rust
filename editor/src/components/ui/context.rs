use std::path::PathBuf;

use egui::{Context, Ui};
use egui_file_dialog::FileDialog;

pub struct UiState {
    pub file_dialog: FileDialog,
    pub current_map: Option<PathBuf>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            file_dialog: FileDialog::new(),
            current_map: None
        }
    }
}

pub struct UiContext {
    renderables: Vec<Box<dyn RenderableUi>>,
    state: UiState
}

impl UiContext {
    pub fn new() -> Self {
        Self {
            renderables: Vec::new(),
            state: UiState::new()
        }
    }

    pub fn add_renderable(&mut self, renderable: impl RenderableUi + 'static) {
        self.renderables.push(Box::new(renderable));
    }

    pub fn run_ui(&mut self, ctx: &Context) {
        for renderable in self.renderables.iter_mut() {
            renderable.ui_with(ctx, &mut self.state)
        }
    }
}

pub trait RenderableUi {
    fn ui_with(&mut self, ctx: &Context, state: &mut UiState);
}