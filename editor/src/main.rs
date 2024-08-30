mod app;
mod components;
mod input_handler;

use app::App;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    pollster::block_on(run());
}

async fn run() {
    const WIDTH: u32 = 640;
    const HEIGHT: u32 = 480;

    let app = App::new(WIDTH, HEIGHT).await;

    app.run().await.unwrap();
}
