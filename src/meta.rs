use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, fmt::Debug};

/// Contains information about a prompt, including its execution details.
#[derive(Serialize, Deserialize, Debug)]
pub struct PromptInfo {
    /// Execution information related to the prompt.
    pub exec_info: ExecInfo,
}

/// Contains execution details such as the remaining queue length.
#[derive(Serialize, Deserialize, Debug)]
pub struct ExecInfo {
    /// The number of remaining tasks in the execution queue.
    pub queue_remaining: usize,
}

/// Represents file information including filename, subfolder, and file type.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct FileInfo {
    /// The name of the file.
    #[serde(alias = "name")]
    pub filename: String,
    /// The subfolder where the file is located.
    pub subfolder: String,
    /// The type of the file.
    pub r#type: String,
}

/// Represents a prompt with an identifier, a number, and potential node errors.
#[derive(Serialize, Deserialize, Debug)]
pub struct PromptStatus {
    /// Unique identifier for the prompt.
    pub prompt_id: String,
    /// A numeric identifier for the prompt.
    pub number: usize,
    /// A mapping of node identifiers to error details in JSON format.
    pub node_errors: HashMap<String, Value>,
}

/// Represents the history of outputs for a prompt.
#[derive(Serialize, Deserialize, Debug)]
pub struct History {
    /// A mapping of output identifiers to their corresponding images.
    pub outputs: HashMap<String, Images>,
}

/// Contains an optional list of image file information.
#[derive(Serialize, Deserialize, Debug)]
pub struct Images {
    /// A vector of file information objects, if available.
    pub images: Option<Vec<FileInfo>>,
}

/// Represents events emitted by the system.
///
/// The enum variants correspond to different event types. The `Unknown` variant
/// holds raw JSON data for unrecognized events.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Event {
    /// A status event containing execution information.
    Status { data: StatusEventData, sid: Option<String> },
    /// A progress event indicating current progress.
    Progress { data: ProgressEventData },
    /// An event indicating that a node has completed execution along with its
    /// output.
    Executed { data: ExecutedEventData },
    /// An event indicating that a node is currently executing.
    Executing { data: ExecutingEventData },
    /// An event signaling the start of execution for a prompt.
    ExecutionStart { data: ExecutionStartEventData },
    /// An event signaling that an error occurred during execution.
    ExecutionError { data: ExecutionErrorEventData },
    /// An event indicating that the execution results were retrieved from the
    /// cache.
    ExecutionCached { data: ExecutionCachedEventData },
    /// An event indicating that the execution was interrupted.
    ExecutionInterrupted { data: ExecutionInterruptedEventData },
    ExecutionSuccess { data: ExecutionSuccessEventData },
    /// An unknown event type that encapsulates raw JSON data.
    #[serde(skip)]
    Unknown(Value),
    /// Events that are not part of the ComfyUI API but are added by the client.
    #[serde(skip)]
    Other(OtherEvent),
}

/// Represents events that are not part of the standard ComfyUI API
/// but are added by the client for additional functionality.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum OtherEvent {
    /// Event indicating a successful reconnection to the WebSocket.
    ReconnectSuccess,
}

/// Represents events that are not part of the standard ComfyUI API
/// but are added by the client for additional functionality.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum OtherEvent {
    /// Event indicating a successful reconnection to the WebSocket.
    ReconnectSuccess,
}

/// Event payload for a status event, containing execution information.
#[derive(Serialize, Deserialize, Debug)]
pub struct StatusEventData {
    /// Execution information associated with the event.
    pub status: StatusEventDataStatus,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StatusEventDataStatus {
    /// Execution information associated with the event.
    pub exec_info: ExecInfo,
}

/// Event payload for a progress update, including current value and maximum
/// value.
#[derive(Serialize, Deserialize, Debug)]
pub struct ProgressEventData {
    /// The current progress value.
    pub value: usize,
    /// The maximum progress value.
    pub max: usize,
}

/// Represents the output of an executed node.
#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    /// A list of image file information objects.
    pub images: Vec<FileInfo>,
}

/// Event payload for a completed execution, including the node identifier,
/// prompt ID, and output data.
#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutedEventData {
    /// Identifier of the node that completed execution.
    pub node: String,
    /// The prompt ID associated with the execution.
    pub prompt_id: String,
    /// The output generated by the executed node.
    pub output: Output,
}

/// Event payload for an execution in progress, including the node identifier
/// and prompt ID.
#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutingEventData {
    /// Identifier of the node currently executing.
    pub node: Option<String>,
    pub display_node: Option<String>,
    /// The prompt ID associated with the execution.
    pub prompt_id: String,
}

/// Event payload indicating that the execution has started.
#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionStartEventData {
    /// The prompt ID for which the execution has started.
    pub prompt_id: String,
    pub timestamp: u64,
}

/// Event payload for an execution error, containing details about the error and
/// its context.
#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionErrorEventData {
    /// The prompt ID associated with the error.
    pub prompt_id: String,
    /// The identifier of the node where the error occurred.
    pub node_id: String,
    /// The type of the node where the error occurred.
    pub node_type: String,
    /// A list of node identifiers that were executed before the error.
    pub executed: Vec<String>,
    /// The error message from the exception.
    pub exception_message: String,
    /// The type of the exception.
    pub exception_type: String,
    /// A traceback of the error as a list of strings.
    pub traceback: Vec<String>,
    /// The current input values at the time of the error.
    pub current_inputs: HashMap<String, Value>,
    /// The current output values at the time of the error.
    pub current_outputs: HashMap<String, Value>,
}

/// Event payload indicating that the execution result was obtained from the
/// cache.
#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionCachedEventData {
    /// A list of node identifiers that were cached.
    pub nodes: Vec<String>,
    /// The prompt ID associated with the cached execution.
    pub prompt_id: String,
    pub timestamp: u64,
}

/// Event payload for an interrupted execution, containing details about the
/// interruption.
#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionInterruptedEventData {
    /// The prompt ID associated with the interruption.
    pub prompt_id: String,
    /// The identifier of the node where the execution was interrupted.
    pub node_id: String,
    /// The type of the node that was interrupted.
    pub node_type: String,
    /// A list of node identifiers that were executed before the interruption.
    pub executed: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionSuccessEventData {
    pub prompt_id: String,
}

/// `Prompt` param for
/// [`ComfyUIClient::post_prompt`](crate::ComfyUIClient::post_prompt).
pub enum Prompt<'a> {
    /// A string slice representing the prompt in JSON format.
    Str(&'a str),
    /// A JSON value representing the prompt data.
    Value(&'a Value),
}

impl<'a> From<&'a str> for Prompt<'a> {
    fn from(value: &'a str) -> Self {
        Prompt::Str(value)
    }
}

impl<'a> From<&'a String> for Prompt<'a> {
    fn from(value: &'a String) -> Self {
        Prompt::Str(value)
    }
}

impl<'a> From<&'a Value> for Prompt<'a> {
    fn from(value: &'a Value) -> Self {
        Prompt::Value(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Tests serialization of different event types.
    #[test]
    fn test_serialize_event() {
        let ev = Event::Status {
            data: StatusEventData {
                status: ExecInfo { queue_remaining: 0 },
            },
        };
        let value = serde_json::to_value(&ev).unwrap();
        assert_eq!(
            value,
            json!({
                "type": "status",
                "data": {
                    "status": {
                        "queue_remaining": 0,
                    }
                }
            })
        );

        let ev = Event::ExecutionStart {
            data: ExecutionStartEventData {
                prompt_id: "xxxxxx".to_string(),
            },
        };
        let value = serde_json::to_value(&ev).unwrap();
        assert_eq!(
            value,
            json!({
                "type": "execution_start",
                "data": {
                    "prompt_id": "xxxxxx",
                }
            })
        );
    }
}
