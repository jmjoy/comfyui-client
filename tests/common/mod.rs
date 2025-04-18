use comfyui_client::{ClientBuilder, ComfyUIClient, EventStream};
use std::sync::Once;

pub fn setup() {
    static START: Once = Once::new();
    START.call_once(|| {
        env_logger::init();
    });
}

pub async fn build_client() -> (ComfyUIClient, EventStream) {
    ClientBuilder::new("http://localhost:8188")
        .build()
        .await
        .unwrap()
}
