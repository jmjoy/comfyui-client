use comfyui_client::{ClientBuilder, ComfyUIClient, EventStream};
use std::sync::Once;
use tokio::task::JoinHandle;

pub fn setup() {
    static START: Once = Once::new();
    START.call_once(|| {
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .init();
    });
}

pub async fn build_client() -> (ComfyUIClient, EventStream, JoinHandle<()>) {
    ClientBuilder::new("http://localhost:8188")
        .build()
        .await
        .unwrap()
}
