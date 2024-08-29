mod app;
mod input_handler;
mod components;

use app::App;
use components::ui::{context::UiContext, left_panel::LeftPanelUi};

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        pollster::block_on(run());
    }
}

async fn run() {
    const WIDTH: u32 = 640;
    const HEIGHT: u32 = 480;

    let mut ui_context = UiContext::new();

    ui_context.add_renderable(LeftPanelUi);

    let app = App::new(WIDTH, HEIGHT, |ctx| ui_context.run_ui(ctx)).await;

    app.run().await.unwrap();
}