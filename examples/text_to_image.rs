use comfyui_client::{ClientBuilder, meta::Event};
use futures_util::StreamExt;
use log::{debug, error, info, warn};
use std::env::temp_dir;
use tokio::fs;
use serde_json::{json, Value};

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
        .filter_module("comfyui_client", log::LevelFilter::Trace)
        .filter_module("text_to_image", log::LevelFilter::Debug)
        .init();

    let (client, mut stream) = ClientBuilder::new("http://localhost:8188")
        .unwrap()
        .build()
        .await
        .unwrap();

    debug!("start to queue prompt");

    let prompt = client.post_prompt(WORKFLOW_JSON).await.unwrap();

    info!(prompt_id:% = prompt.prompt_id; "queued prompt");

    while let Some(ev) = stream.next().await {
        let ev = ev.unwrap();
        match ev {
            Event::Status { data, sid } => {
                debug!(data:?, sid:?; "receive status event");
            }
            Event::ExecutionStart { data, .. } => {
                debug!(data:?; "receive execution status event");
            }
            Event::ExecutionCached { data, .. } => {
                debug!(data:?; "receive execution cached event");
            }
            Event::Progress { data, .. } => {
                debug!(data:?; "receive process event");
            }
            Event::Executing { data, .. } => {
                debug!(data:?; "receive executing event");
            }
            Event::Executed { data, .. } => {
                debug!(data:?; "receive executed event");

                for image in data.output.images {
                    let buf = client.get_view(&image).await.unwrap();
                    let file_path = temp_dir().join(image.filename);
                    fs::write(&file_path, &buf).await.unwrap();
                    info!(file_path:% = file_path.display(); "write to file success");
                }
            }
            Event::ExecutionSuccess { data, .. } => {
                debug!(data:?; "receive execution success event");
                break;
            }
            Event::ExecutionError { data, .. } => {
                error!(data:?; "receive execution error event");
            }
            Event::ExecutionInterrupted { data, .. } => {
                error!(execution_interrupted_event:? = data; "receive execution_interrupted_event");
            }
            Event::Unknown(event) => {
                if event["type"] != json!("crystools.monitor") {
                    warn!(event:?; "receive unknown event");
                }
            }
            Event::Other(event) => {
                warn!(event:?; "receive other event");
            }
            _ => {
                warn!("receive unhandled event type");
            }
            Event::Other(ev) => {
                warn!(ev:?; "receive other event");
            }
            _ => {}
        }
    }
}
