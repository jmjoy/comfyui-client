#![warn(rust_2018_idioms, missing_docs)]
#![warn(clippy::dbg_macro, clippy::print_stdout)]
#![doc = include_str!("../README.md")]

/// Module containing error definitions.
pub mod errors;
/// Module containing metadata such as prompt and file information.
pub mod meta;

pub use crate::errors::{ClientError, ClientResult};
use crate::meta::{FileInfo, PromptInfo};
use bytes::Bytes;
use cfg_if::cfg_if;
use errors::{ApiBody, ApiError};
use futures_util::StreamExt;
use log::trace;
use meta::{Event, History, OtherEvent, Prompt, PromptStatus};
use reqwest::{
    Body, IntoUrl, Response,
    multipart::{self},
};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};
use tokio::{net::TcpStream, sync::mpsc, task::JoinHandle};
use tokio_stream::wrappers::ReceiverStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{self, Message},
};
use url::Url;
use uuid::Uuid;

/// A builder for creating a [`ComfyUIClient`] instance.
///
/// This builder helps initialize the client with the provided base URL and sets
/// up a websocket connection to stream events.
pub struct ClientBuilder {
    base_url: Url,
    channel_bound: usize,
    reconnect_web_socket: bool,
}

impl ClientBuilder {
    /// Creates a new [`ClientBuilder`] instance.
    ///
    /// # Parameters
    ///
    /// - `base_url`: The base URL of the ComfyUI service.
    ///
    /// # Returns
    ///
    /// A new instance of [`ClientBuilder`] wrapped in a `ClientResult`, or an
    /// error if the URL is invalid.
    pub fn new(base_url: impl IntoUrl) -> ClientResult<Self> {
        Ok(Self {
            base_url: base_url.into_url()?,
            channel_bound: 100,
            reconnect_web_socket: true,
        })
    }

    /// Sets the capacity of the internal channel used for event streaming.
    ///
    /// This controls how many events can be buffered before backpressure is
    /// applied. The default value is 100.
    ///
    /// # Parameters
    ///
    /// - `channel_bound`: The maximum number of events the channel can hold.
    ///
    /// # Returns
    ///
    /// The updated [`ClientBuilder`] instance.
    pub fn channel_bound(mut self, channel_bound: usize) -> Self {
        self.channel_bound = channel_bound;
        self
    }

    /// Sets whether the websocket should attempt to reconnect automatically
    /// when disconnected.
    ///
    /// By default, reconnection is enabled (`true`).
    ///
    /// # Parameters
    ///
    /// - `reconnect`: Whether to attempt reconnection when the WebSocket
    ///   connection drops unexpectedly.
    ///
    /// # Returns
    ///
    /// The updated [`ClientBuilder`] instance.
    pub fn reconnect_web_socket(mut self, reconnect: bool) -> Self {
        self.reconnect_web_socket = reconnect;
        self
    }

    /// Helper method to establish a WebSocket connection
    async fn connect_to_websocket(
        ws_url: &Url,
    ) -> ClientResult<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        let (stream, _) = if ws_url.scheme() == "wss" {
            cfg_if! {
                if #[cfg(feature = "rustls")] {
                    let root_store = rustls::RootCertStore {
                        roots: webpki_roots::TLS_SERVER_ROOTS.into(),
                    };
                    let config = rustls::ClientConfig::builder()
                        .with_root_certificates(root_store)
                        .with_no_client_auth();

                    tokio_tungstenite::connect_async_tls_with_config(
                        ws_url.clone(),
                        None,
                        false,
                        Some(tokio_tungstenite::Connector::Rustls(std::sync::Arc::new(config))),
                    )
                    .await?
                } else {
                    connect_async(ws_url.clone()).await?
                }
            }
        } else {
            connect_async(ws_url.clone()).await?
        };

        Ok(stream)
    }

    /// Builds the [`ComfyUIClient`] along with an associated [`EventStream`].
    ///
    /// This method establishes a websocket connection and spawns an
    /// asynchronous task to process incoming messages.
    ///
    /// # Returns
    ///
    /// A tuple containing the [`ComfyUIClient`] and [`EventStream`] on success,
    /// or an error.
    pub async fn build(self) -> ClientResult<(ComfyUIClient, EventStream)> {
        let base_url = self.base_url;
        let http_client = reqwest::Client::new();
        let client_id = Uuid::new_v4().to_string();
        let reconnect_web_socket = self.reconnect_web_socket;

        let (ev_tx, ev_rx) = mpsc::channel(self.channel_bound);

        let ws_url = Self::generate_websocket_url(base_url.clone(), &client_id)?;

        // Initial connection
        let ws_stream = Self::connect_to_websocket(&ws_url).await?;

        // Spawn the stream handling task with reconnection support
        let stream_handle = tokio::spawn(async move {
            let (_, mut read_stream) = ws_stream.split();

            loop {
                let mut connection_alive = true;

                // Process messages until the connection drops
                while let Some(msg) = read_stream.next().await {
                    match msg {
                        Ok(message) => {
                            let ev = EventStream::handle_message(Ok(message));
                            let Some(ev) = ev.transpose() else {
                                continue;
                            };
                            if ev_tx.send(ev).await.is_err() {
                                connection_alive = false;
                                break;
                            }
                        }
                        Err(err) => {
                            // Connection error occurred
                            connection_alive = false;
                            // Send error to the stream
                            let _ = ev_tx.send(Err(ClientError::from(err))).await;
                            break;
                        }
                    }
                }

                // If reconnect is disabled or the channel is closed, exit the loop
                if !reconnect_web_socket || ev_tx.is_closed() {
                    break;
                }

                // Attempt to reconnect with a small delay until successful or channel closed
                if !connection_alive {
                    // Keep trying to reconnect until successful
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                        // Check if channel is closed before attempting reconnection
                        if ev_tx.is_closed() {
                            break;
                        }

                        // Try to establish a new connection
                        match Self::connect_to_websocket(&ws_url).await {
                            Ok(new_stream) => {
                                // Successfully reconnected
                                (_, read_stream) = new_stream.split();
                                // Send reconnection success event
                                let _ = ev_tx
                                    .send(Ok(Event::Other(OtherEvent::ReconnectSuccess)))
                                    .await;
                                // Break out of the reconnection loop and continue with the new
                                // connection
                                break;
                            }
                            Err(err) => {
                                // Failed to reconnect, send error and continue trying
                                let _ = ev_tx.send(Err(err)).await;
                                // If channel closed during error sending, exit
                                if ev_tx.is_closed() {
                                    break;
                                }
                                // Otherwise continue the reconnection loop
                            }
                        }
                    }

                    // If the channel was closed during reconnection attempts, exit the main loop
                    if ev_tx.is_closed() {
                        break;
                    }
                } else {
                    // Exit when connection is closed normally without errors
                    break;
                }
            }
        });

        let rx_stream = ReceiverStream::new(ev_rx);

        let client = ComfyUIClient {
            base_url,
            http_client,
            client_id,
        };

        let stream = EventStream {
            stream_handle,
            rx_stream,
        };

        Ok((client, stream))
    }

    /// Builds a [`ComfyUIClient`] instance configured for HTTP-only
    /// communication.
    ///
    /// This method initializes the client without establishing a websocket
    /// connection, enabling you to interact with the ComfyUI service using
    /// only HTTP (REST) requests.
    ///
    /// # Returns
    ///
    /// A [`ComfyUIClient`] instance on success, or an error.
    pub async fn build_only_http(self) -> ClientResult<ComfyUIClient> {
        let base_url = self.base_url;
        let http_client = reqwest::Client::new();
        let client_id = Uuid::new_v4().to_string();

        Ok(ComfyUIClient {
            base_url,
            http_client,
            client_id,
        })
    }

    /// Generates the websocket URL based on the base URL and client ID.
    ///
    /// This method changes the URL scheme to `wss` if the base URL uses HTTPS,
    /// or `ws` otherwise, appends the `ws` path, and adds a query parameter
    /// for `clientId`.
    ///
    /// # Parameters
    ///
    /// - `base_url`: The base URL of the ComfyUI service.
    /// - `client_id`: The unique identifier for the client.
    ///
    /// # Returns
    ///
    /// The generated websocket URL on success, or an error if the URL cannot be
    /// modified.
    fn generate_websocket_url(base_url: Url, client_id: &str) -> ClientResult<Url> {
        let mut ws_url = base_url;
        let scheme = if ws_url.scheme() == "https" {
            "wss"
        } else {
            "ws"
        };
        ws_url
            .set_scheme(scheme)
            .map_err(|_| ClientError::SetWsScheme)?;
        ws_url = ws_url.join("ws")?;
        ws_url.query_pairs_mut().append_pair("clientId", client_id);
        Ok(ws_url)
    }
}

/// A client for interacting with the ComfyUI service.
///
/// This client provides methods to fetch history, prompts, views, and to upload
/// images.
pub struct ComfyUIClient {
    client_id: String,
    base_url: Url,
    http_client: reqwest::Client,
}

impl ComfyUIClient {
    /// Retrieves the history for a specified prompt.
    ///
    /// Sends a GET request to the `history/{prompt_id}` endpoint and parses the
    /// returned history data.
    ///
    /// # Parameters
    ///
    /// - `prompt_id`: The ID of the prompt whose history is being requested.
    ///
    /// # Returns
    ///
    /// An optional [`History`] object wrapped in a `ClientResult`. Returns
    /// `None` if the history is not found.
    pub async fn get_history(&self, prompt_id: &str) -> ClientResult<Option<History>> {
        let resp = self
            .http_client
            .get(self.base_url.join(&format!("history/{prompt_id}"))?)
            .send()
            .await?;
        let resp = Self::error_for_status(resp).await?;
        let mut histories = resp.json::<HashMap<String, History>>().await?;
        Ok(histories.remove(prompt_id))
    }

    /// Retrieves the current prompt information.
    ///
    /// Sends a GET request to the `prompt` endpoint and returns the parsed
    /// [`PromptInfo`] data.
    ///
    /// # Returns
    ///
    /// A [`PromptInfo`] object on success, or an error.
    pub async fn get_prompt(&self) -> ClientResult<PromptInfo> {
        let resp = self
            .http_client
            .get(self.base_url.join("prompt")?)
            .send()
            .await?;
        let resp = Self::error_for_status(resp).await?;
        Ok(resp.json().await?)
    }

    /// Retrieves view data corresponding to the provided file information.
    ///
    /// Sends a GET request to the `view` endpoint, including the file
    /// information as query parameters.
    ///
    /// # Parameters
    ///
    /// - `file_info`: A [`FileInfo`] object containing details about the file.
    ///
    /// # Returns
    ///
    /// The response as a [`Bytes`] object on success, or an error.
    pub async fn get_view(&self, file_info: &FileInfo) -> ClientResult<Bytes> {
        let resp = self
            .http_client
            .get(self.base_url.join("view")?)
            .query(file_info)
            .send()
            .await?;
        let resp = Self::error_for_status(resp).await?;
        Ok(resp.bytes().await?)
    }

    /// Sends a prompt in JSON format.
    ///
    /// Constructs the request payload (including the client ID and prompt data)
    /// and sends a POST request to the `prompt` endpoint.
    ///
    /// # Parameters
    ///
    /// - `prompt`: representing the prompt data.
    ///
    /// # Returns
    ///
    /// A [`PromptStatus`] object on success, or an error.
    pub async fn post_prompt(&self, prompt: impl Into<Prompt<'_>>) -> ClientResult<PromptStatus> {
        let prompt = match prompt.into() {
            Prompt::Str(prompt) => &serde_json::from_str::<Value>(prompt)?,
            Prompt::Value(prompt) => prompt,
        };
        let data = json!({"client_id": &self.client_id, "prompt": prompt});
        let resp = self
            .http_client
            .post(self.base_url.join("prompt")?)
            .json(&data)
            .send()
            .await?;
        let resp = Self::error_for_status(resp).await?;
        Ok(resp.json().await?)
    }

    /// Uploads an image.
    ///
    /// Constructs a multipart form containing the image data and file
    /// information, then sends a POST request to the `upload/image` endpoint.
    ///
    /// # Parameters
    ///
    /// - `body`: The image data, convertible into a [`Body`].
    /// - `info`: A [`FileInfo`] object containing details about the image file.
    /// - `overwrite`: A boolean indicating whether to overwrite an existing
    ///   file.
    ///
    /// # Returns
    ///
    /// An updated [`FileInfo`] object on success, or an error.
    pub async fn upload_image(
        &self, body: impl Into<Body>, info: &FileInfo, overwrite: bool,
    ) -> ClientResult<FileInfo> {
        let part = multipart::Part::stream(body).file_name(info.filename.to_string());
        let mut form = multipart::Form::new()
            .part("image", part)
            .text("overwrite", overwrite.to_string())
            .text("type", info.r#type.to_string());
        if !info.subfolder.is_empty() {
            form = form.text("subfolder", info.subfolder.to_string());
        }

        let resp = self
            .http_client
            .post(self.base_url.join("upload/image")?)
            .multipart(form)
            .send()
            .await?;

        let resp = Self::error_for_status(resp).await?;
        Ok(resp.json().await?)
    }

    /// Checks the HTTP response status code and returns an error if it
    /// indicates failure.
    ///
    /// If the response status is a client or server error, this method attempts
    /// to parse the response body as JSON. If parsing fails, it returns the
    /// body as text.
    ///
    /// # Parameters
    ///
    /// - `resp`: The HTTP response to evaluate.
    ///
    /// # Returns
    ///
    /// The original response if the status is successful, or an error if the
    /// status indicates a failure.
    async fn error_for_status(resp: Response) -> ClientResult<Response> {
        let status = resp.status();
        if status.is_client_error() || status.is_server_error() {
            let body = resp.text().await?;
            let body = match serde_json::from_str::<Value>(&body) {
                Ok(value) => ApiBody::Json(value),
                Err(_) => ApiBody::Text(body),
            };
            Err(ApiError { status, body }.into())
        } else {
            Ok(resp)
        }
    }
}

/// A structure representing the event stream received via a websocket
/// connection.
///
/// This stream continuously processes events from the ComfyUI service.
pub struct EventStream {
    stream_handle: JoinHandle<()>,
    rx_stream: ReceiverStream<ClientResult<Event>>,
}

impl EventStream {
    /// Handles a single websocket message and attempts to parse it as an
    /// [`Event`].
    ///
    /// For text messages, it tries to deserialize the message into an
    /// [`Event`]. If the deserialization fails, it wraps the message as
    /// [`Event::Unknown`]. Other message types are ignored.
    ///
    /// # Parameters
    ///
    /// - `msg`: A result containing a [`Message`] from the websocket.
    ///
    /// # Returns
    ///
    /// An `Option<Event>` wrapped in a `ClientResult`. Returns `None` for
    /// unsupported message types.
    fn handle_message(msg: tungstenite::Result<Message>) -> ClientResult<Option<Event>> {
        let msg = msg?;
        match msg {
            Message::Text(b) => {
                trace!(message:% = b.as_str(); "received websocket message");
                let value = serde_json::from_slice::<Value>(b.as_bytes())?;
                match serde_json::from_value::<Event>(value.clone()) {
                    Ok(ev) => Ok(Some(ev)),
                    Err(_) => Ok(Some(Event::Unknown(value))),
                }
            }
            _ => Ok(None),
        }
    }
}

impl Drop for EventStream {
    /// When the [`EventStream`] is dropped, abort the associated websocket
    /// handling task.
    fn drop(&mut self) {
        self.stream_handle.abort();
    }
}

impl Deref for EventStream {
    type Target = ReceiverStream<ClientResult<Event>>;

    /// Allows access to the inner [`ReceiverStream`] containing the events.
    fn deref(&self) -> &Self::Target {
        &self.rx_stream
    }
}

impl DerefMut for EventStream {
    /// Allows mutable access to the inner [`ReceiverStream`].
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rx_stream
    }
}
