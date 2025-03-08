use comfyui_client::{ClientBuilder, meta::Event};
use futures_util::StreamExt;
use log::{debug, error, info, warn};
use std::env::temp_dir;
use tokio::fs;

const WORKFLOW_JSON: &str = r#"
{
  "6": {
    "inputs": {
      "text": "cute anime girl with massive fluffy fennec ears and a big fluffy tail blonde messy long hair blue eyes wearing a maid outfit with a long black gold leaf pattern dress and a white apron mouth open placing a fancy black forest cake with candles on top of a dinner table of an old dark Victorian mansion lit by candlelight with a bright window to the foggy forest and very expensive stuff everywhere there are paintings on the walls",
      "clip": [
        "30",
        1
      ]
    },
    "class_type": "CLIPTextEncode",
    "_meta": {
      "title": "CLIP Text Encode (Positive Prompt)"
    }
  },
  "8": {
    "inputs": {
      "samples": [
        "31",
        0
      ],
      "vae": [
        "30",
        2
      ]
    },
    "class_type": "VAEDecode",
    "_meta": {
      "title": "VAE解码"
    }
  },
  "27": {
    "inputs": {
      "width": 1024,
      "height": 1024,
      "batch_size": 1
    },
    "class_type": "EmptySD3LatentImage",
    "_meta": {
      "title": "空Latent图像（SD3）"
    }
  },
  "30": {
    "inputs": {
      "ckpt_name": "flux1-dev-fp8.safetensors"
    },
    "class_type": "CheckpointLoaderSimple",
    "_meta": {
      "title": "Checkpoint加载器（简易）"
    }
  },
  "31": {
    "inputs": {
      "seed": 450323191819485,
      "steps": 20,
      "cfg": 1,
      "sampler_name": "euler",
      "scheduler": "simple",
      "denoise": 1,
      "model": [
        "30",
        0
      ],
      "positive": [
        "35",
        0
      ],
      "negative": [
        "33",
        0
      ],
      "latent_image": [
        "27",
        0
      ]
    },
    "class_type": "KSampler",
    "_meta": {
      "title": "K采样器"
    }
  },
  "33": {
    "inputs": {
      "text": "",
      "clip": [
        "30",
        1
      ]
    },
    "class_type": "CLIPTextEncode",
    "_meta": {
      "title": "CLIP Text Encode (Negative Prompt)"
    }
  },
  "35": {
    "inputs": {
      "guidance": 3.5,
      "conditioning": [
        "6",
        0
      ]
    },
    "class_type": "FluxGuidance",
    "_meta": {
      "title": "Flux引导"
    }
  },
  "38": {
    "inputs": {
      "images": [
        "8",
        0
      ]
    },
    "class_type": "PreviewImage",
    "_meta": {
      "title": "预览图像"
    }
  }
}
    "#;

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let (client, mut stream) = ClientBuilder::new("http://localhost:8188")
        .unwrap()
        .build()
        .await
        .unwrap();

    debug!("start to queue prompt");

    let prompt = client.post_prompt_str(WORKFLOW_JSON).await.unwrap();

    info!(prompt_id:% = prompt.prompt_id; "queued prompt");

    while let Some(ev) = stream.next().await {
        let ev = ev.unwrap();
        match ev {
            Event::Status(status_event) => {
                info!(status_event:?; "receive status event");
            }
            Event::ExecutionStart(execution_start_event) => {
                info!(execution_start_event:?; "receive execution status event");
            }
            Event::ExecutionCached(execution_cached_event) => {
                info!(execution_cached_event:?; "receive execution cached event");
            }
            Event::Progress(progress_event) => {
                info!(progress_event:?; "receive process event");
            }
            Event::Executing(executing_event) => {
                info!(executing_event:?; "receive executing event");
            }
            Event::Executed(executed_event) => {
                info!(executed_event:?; "receive executed event");

                for image in executed_event.output.images {
                    let buf = client.get_view(&image).await.unwrap();
                    let file_path = temp_dir().join(image.filename);
                    fs::write(&file_path, &buf).await.unwrap();
                    info!(file_path:% = file_path.display(); "write to file success");
                }
            }
            Event::ExecutionError(execution_error_event) => {
                error!(execution_error_event:?; "receive execution error event");
            }
            Event::ExecutionInterrupted(execution_interrupted_event) => {
                error!(execution_interrupted_event:?; "receive execution_interrupted_event");
            }
            Event::Unknown(value) => {
                warn!(event:? = value; "receive unknown event");
            }
        }
    }
}
