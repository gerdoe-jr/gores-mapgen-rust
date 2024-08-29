use egui::Context;

pub struct UiContext {
    renderables: Vec<Box<dyn RenderableUi>>,
}

impl UiContext {
    pub fn new() -> Self {
        Self {
            renderables: Vec::new(),
        }
    }

    pub fn add_renderable(&mut self, renderable: impl RenderableUi + 'static) {
        self.renderables.push(Box::new(renderable));
    }

    fn run_ui(&mut self, ctx: &Context) {
        for renderable in self.renderables.iter_mut() {
            renderable.ui_with(ctx);
        }
    }

    pub fn runner(&mut self) -> impl FnMut(&Context) + '_ {
        |ctx| self.run_ui(ctx)
    }
}

pub trait RenderableUi {
    fn ui_with(&mut self, ctx: &Context);
}