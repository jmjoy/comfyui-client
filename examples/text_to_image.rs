use comfyui_client::{
    ClientBuilder,
    meta::{ComfyEvent, Event},
};
use futures_util::StreamExt;
use log::{debug, error, info, warn};
use serde_json::json;
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
      "title": "VAE Decode"
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
      "title": "EmptySD3LatentImage"
    }
  },
  "30": {
    "inputs": {
      "ckpt_name": "flux1-schnell-fp8.safetensors"
    },
    "class_type": "CheckpointLoaderSimple",
    "_meta": {
      "title": "Load Checkpoint"
    }
  },
  "31": {
    "inputs": {
      "seed": 173805153958730,
      "steps": 4,
      "cfg": 1,
      "sampler_name": "euler",
      "scheduler": "simple",
      "denoise": 1,
      "model": [
        "30",
        0
      ],
      "positive": [
        "6",
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
      "title": "KSampler"
    }
  },
  "33": {
    "inputs": {
      "text": "",
      "speak_and_recognation": {
        "__value__": [
          false,
          true
        ]
      },
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
  "38": {
    "inputs": {
      "images": [
        "8",
        0
      ]
    },
    "class_type": "PreviewImage",
    "_meta": {
      "title": "Preview Image"
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
        .build()
        .await
        .unwrap();

    debug!("start to queue prompt");

    let prompt = client.post_prompt(WORKFLOW_JSON, None).await.unwrap();

    info!(prompt_id:% = prompt.prompt_id; "queued prompt");

    while let Some(ev) = stream.next().await {
        let ev = ev.unwrap();
        match ev {
            Event::Comfy(comfy_event) => match comfy_event {
                ComfyEvent::Status { data, sid } => {
                    debug!(data:?, sid:?; "receive status event");
                }
                ComfyEvent::ExecutionStart { data } => {
                    debug!(data:?; "receive execution status event");
                }
                ComfyEvent::ExecutionCached { data } => {
                    debug!(data:?; "receive execution cached event");
                }
                ComfyEvent::Progress { data } => {
                    debug!(data:?; "receive process event");
                }
                ComfyEvent::ProgressState { data } => {
                    debug!(data:?; "receive process state event");
                }
                ComfyEvent::Executing { data } => {
                    debug!(data:?; "receive executing event");
                }
                ComfyEvent::Executed { data } => {
                    debug!(data:?; "receive executed event");

                    if let Some(output) = &data.output {
                        if let Some(images) = &output.images {
                            for image in images {
                                let buf = client.get_view(image).await.unwrap();
                                let file_path = temp_dir().join(&image.filename);
                                fs::write(&file_path, &buf).await.unwrap();
                                info!(file_path:% = file_path.display(); "write to file success");
                            }
                        }
                    }
                }
                ComfyEvent::ExecutionSuccess { data } => {
                    debug!(data:?; "receive execution success event");
                    break;
                }
                ComfyEvent::ExecutionError { data } => {
                    error!(data:?; "receive execution error event");
                    break;
                }
                ComfyEvent::ExecutionInterrupted { data } => {
                    error!(data:?; "receive execution_interrupted_event");
                    break;
                }
                ComfyEvent::Unknown(event) => {
                    // exclude monitoring events of `ComfyUI-Crystools` plugin
                    if event["type"] != json!("crystools.monitor") {
                        warn!(event:?; "receive unknown comfy event");
                    }
                }
            },
            Event::Connection(event) => {
                warn!(event:?; "receive connection event");
            }
            _ => {
                warn!("receive unhandled event type");
            }
        }
    }
}
