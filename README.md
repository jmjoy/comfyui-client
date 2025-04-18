# ComfyUI Client

[![Rust Version](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)
[![GitHub Actions CI](https://img.shields.io/github/actions/workflow/status/jmjoy/comfyui-client/ci.yml?branch=master&label=CI&logo=github)](https://github.com/jmjoy/comfyui-client/actions)
[![crates.io](https://img.shields.io/crates/v/comfyui-client.svg)](https://crates.io/crates/comfyui-client)
[![Downloads](https://img.shields.io/crates/d/comfyui-client.svg)](https://crates.io/crates/comfyui-client)
[![docs.rs](https://img.shields.io/docsrs/comfyui-client?logo=rust)](https://docs.rs/comfyui-client)
[![License](https://img.shields.io/crates/l/comfyui-client?color=blue)](https://github.com/jmjoy/comfyui-client/blob/master/LICENSE)

Rust client for [ComfyUI](https://github.com/comfyanonymous/ComfyUI).

## API Reference

The following table lists the available APIs in ComfyUIClient:

| Method | URL | Purpose | Client Method |
|--------|-----|---------|---------------|
| GET | `/history/{prompt_id}` | Retrieves the history for a specified prompt | `get_history` |
| GET | `/prompt` | Retrieves the current prompt information | `get_prompt` |
| GET | `/view` | Retrieves view data for a file (e.g., images) | `get_view` |
| POST | `/prompt` | Sends a prompt in JSON format | `post_prompt` |
| POST | `/upload/image` | Uploads an image to ComfyUI | `upload_image` |

Additionally, the client establishes a WebSocket connection to `/ws` to receive real-time events from ComfyUI.

## Examples

Refer to [examples](https://github.com/jmjoy/comfyui-client/tree/master/examples).

## License

MulanPSL-2.0
