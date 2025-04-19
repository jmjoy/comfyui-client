mod common;

use bytes::Bytes;
use comfyui_client::meta::{Event, ExecutedOutput, FileInfo};
use tokio::fs::{self, File};
use tokio_stream::StreamExt;

#[tokio::test]
async fn test_get_prompt() {
    common::setup();
    let (client, _) = common::build_client().await;
    client.get_prompt().await.unwrap();
}

#[tokio::test]
async fn test_integration() {
    common::setup();
    let (client, mut stream) = common::build_client().await;

    let file = File::open("./tests/data/cat.webp").await.unwrap();
    let file_info = FileInfo {
        filename: "cat.webp".to_string(),
        subfolder: "".to_string(),
        r#type: "input".to_string(),
    };
    let result_info = client.upload_image(file, &file_info, true).await.unwrap();
    assert_eq!(result_info, file_info);

    let workflow_json = fs::read_to_string("./tests/data/blur-cat-workflow.json")
        .await
        .unwrap();
    let prompt = client.post_prompt(&workflow_json).await.unwrap();

    let mut image_buf = Bytes::new();

    'stream: while let Some(ev) = stream.next().await {
        let ev = ev.unwrap();

        if let Event::Executed { data } = ev {
            let Some(ExecutedOutput::Images(images)) = data.output else {
                continue;
            };
            let images = images.images;
            assert_eq!(images.len(), 1);

            image_buf = client.get_view(&images[0]).await.unwrap();
            break 'stream;
        }
    }

    let mut history = client
        .get_history(&prompt.prompt_id)
        .await
        .unwrap()
        .unwrap();
    let image = history
        .outputs
        .remove("5")
        .unwrap()
        .images
        .unwrap()
        .remove(0);

    let image2_buf = client.get_view(&image).await.unwrap();

    assert_eq!(image_buf, image2_buf);
}

#[tokio::test]
async fn test_integration_with_stream_ext() {
    common::setup();
    let (client, stream) = common::build_client().await;

    let mut stream = stream
        .filter_map(|result| result.ok())
        .filter_map(|ev| {
            if let Event::Executed { data } = ev {
                if let Some(ExecutedOutput::Images(images)) = data.output {
                    Some(images.images)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .take(1);

    let file = File::open("./tests/data/girl.webp").await.unwrap();
    let file_info = FileInfo {
        filename: "girl.webp".to_string(),
        subfolder: "".to_string(),
        r#type: "input".to_string(),
    };
    let result_info = client.upload_image(file, &file_info, true).await.unwrap();
    assert_eq!(result_info, file_info);

    let workflow_json = fs::read_to_string("./tests/data/scale-girl-workflow.json")
        .await
        .unwrap();
    client.post_prompt(&workflow_json).await.unwrap();

    let mut files = Vec::new();
    while let Some(message) = stream.next().await {
        files.push(message);
    }

    assert_eq!(files.len(), 1);
    assert_eq!(files[0][0].r#type, "temp");
}
