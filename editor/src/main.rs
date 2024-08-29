mod app;
mod input_handler;
mod components;

use app::App;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        pollster::block_on(run());
    }
}

async fn run() {
    const WIDTH: u32 = 640;
    const HEIGHT: u32 = 480;

    let app = App::new(WIDTH, HEIGHT).await;

    app.run().await.unwrap();
}