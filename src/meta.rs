use crate::ClientError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, fmt::Debug};
use tokio_tungstenite::tungstenite;

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

/// Represents events emitted by the ComfyUI client during workflow execution.
///
/// This structure allows for clear separation between service-level events and
/// client-side connection management events.
#[non_exhaustive]
pub enum Event {
    /// `Comfy` events originate from the ComfyUI service itself
    Comfy(ComfyEvent),
    /// `Connection` events relate to WebSocket connection management
    Connection(ConnectionEvent),
}

/// Represents events emitted by the ComfyUI service during workflow execution.
///
/// This enum encapsulates various event types that occur during the lifecycle
/// of a workflow, from queuing to completion. Each variant contains specific
/// data relevant to that event type. The `Unknown` variant captures any
/// unrecognized events from the API.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ComfyEvent {
    /// A status event containing queue and execution information.
    Status {
        /// Data payload for the status event, including execution information.
        data: StatusEventData,
        /// Optional session identifier associated with this event.
        sid: Option<String>,
    },
    /// A progress event indicating current progress of an operation.
    Progress {
        /// Data payload containing current and maximum progress values.
        data: ProgressEventData,
    },
    /// An event indicating that a node has completed execution along with its
    /// output data.
    Executed {
        /// Data payload containing node output information and associated
        /// images.
        data: ExecutedEventData,
    },
    /// An event indicating that a node is currently executing.
    Executing {
        /// Data payload identifying the currently executing node.
        data: ExecutingEventData,
    },
    /// An event signaling the start of execution for a prompt.
    ExecutionStart {
        /// Data payload containing prompt ID and timestamp information.
        data: ExecutionStartEventData,
    },
    /// An event signaling that an error occurred during execution.
    ExecutionError {
        /// Data payload containing detailed error information and context.
        data: ExecutionErrorEventData,
    },
    /// An event indicating that the execution results were retrieved from the
    /// cache.
    ExecutionCached {
        /// Data payload containing information about which nodes were retrieved
        /// from cache.
        data: ExecutionCachedEventData,
    },
    /// An event indicating that the execution was interrupted.
    ExecutionInterrupted {
        /// Data payload containing details about where and why the execution
        /// was interrupted.
        data: ExecutionInterruptedEventData,
    },
    /// An event indicating that the entire workflow has executed successfully.
    ExecutionSuccess {
        /// Data payload containing the prompt ID of the successfully executed
        /// workflow.
        data: ExecutionSuccessEventData,
    },
    /// An unknown event type that encapsulates raw JSON data for events not
    /// explicitly defined.
    #[serde(skip)]
    Unknown(Value),
}

/// Represents events that are not part of the standard ComfyUI API
/// but are added by the client for additional functionality.
///
/// These events are used internally by the client to handle WebSocket
/// connection management and error reporting, allowing the application to
/// respond to connection-related events that aren't part of the ComfyUI
/// protocol.
#[derive(Debug)]
#[non_exhaustive]
pub enum ConnectionEvent {
    /// Event indicating a successful reconnection to the WebSocket.
    ///
    /// Emitted when the client successfully reestablishes a connection after
    /// a disconnection.
    WSReconnectSuccess,

    /// Event containing an error that occurred during a reconnection attempt.
    ///
    /// Provides detailed error information about why a reconnection attempt
    /// failed, allowing clients to implement appropriate retry or fallback
    /// strategies.
    WSReconnectError(ClientError),

    /// Event containing an error that occurred while receiving messages.
    ///
    /// Indicates that an error occurred in the WebSocket communication channel
    /// while trying to receive messages from the ComfyUI server.
    WSReceiveError(tungstenite::Error),
}

/// Event payload for a status event, containing execution information.
///
/// This structure is received when ComfyUI sends a status update, typically
/// containing information about the current execution queue state.
#[derive(Serialize, Deserialize, Debug)]
pub struct StatusEventData {
    /// Execution information associated with the event, including queue
    /// details.
    pub status: StatusEventStatus,
}

/// Container for execution information within a status event.
///
/// Holds detailed execution information about the current state of the ComfyUI
/// service, such as the number of remaining items in the execution queue.
#[derive(Serialize, Deserialize, Debug)]
pub struct StatusEventStatus {
    /// Execution information including queue status and other execution
    /// metrics.
    pub exec_info: ExecInfo,
}

/// Event payload for a progress update, including current value and maximum
/// value.
///
/// This structure is received when ComfyUI reports progress of an operation,
/// such as image generation or processing.
#[derive(Serialize, Deserialize, Debug)]
pub struct ProgressEventData {
    /// The current progress value representing the completed steps.
    pub value: usize,
    /// The maximum progress value representing the total number of steps.
    pub max: usize,
}

/// Represents the output of an executed node.
///
/// Contains the results produced by a node in the workflow after successful
/// execution. This can include generated or processed images in the `images`
/// field, as well as other arbitrary output data in the `others` map.
#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutedOutput {
    /// Optional list of image file information objects generated or processed
    /// by the node.
    pub images: Option<Vec<FileInfo>>,
    /// Additional output data that doesn't fit into predefined categories.
    #[serde(flatten)]
    pub others: HashMap<String, Value>,
}

/// Event payload for a completed execution, including the node identifier,
/// prompt ID, and output data.
///
/// This structure is received when a specific node in the workflow completes
/// execution and produces output, such as generated images.
#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutedEventData {
    /// Identifier of the node that completed execution.
    pub node: String,
    /// The prompt ID associated with the execution.
    pub prompt_id: String,
    /// The output generated by the executed node, containing resulting images
    /// or other data.
    pub output: Option<ExecutedOutput>,
}

/// Event payload for an execution in progress, including the node identifier
/// and prompt ID.
///
/// This structure is received when ComfyUI begins executing a specific node in
/// the workflow. It provides information about which node is currently being
/// processed.
#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutingEventData {
    /// Identifier of the node currently executing. May be None in certain
    /// cases.
    pub node: Option<String>,
    /// Optional display name of the executing node, providing a more
    /// user-friendly identifier.
    pub display_node: Option<String>,
    /// The prompt ID associated with the execution, linking this event to a
    /// specific workflow run.
    pub prompt_id: String,
}

/// Event payload indicating that the execution has started.
///
/// This structure is received when ComfyUI begins executing a workflow.
/// It serves as an initial notification that the workflow processing has begun
/// and provides timing information for performance tracking.
#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionStartEventData {
    /// The prompt ID for which the execution has started, identifying the
    /// workflow run.
    pub prompt_id: String,
    /// Unix timestamp indicating when the execution started, useful for timing
    /// analysis.
    pub timestamp: u64,
}

/// Event payload for an execution error, containing details about the error and
/// its context.
///
/// This structure is received when an error occurs during workflow execution.
/// It provides comprehensive information about the error, including where it
/// occurred and the state of inputs and outputs at the time of the error.
#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionErrorEventData {
    /// The prompt ID associated with the error, identifying the workflow
    /// execution.
    pub prompt_id: String,
    /// The identifier of the node where the error occurred within the workflow.
    pub node_id: String,
    /// The type of the node where the error occurred (e.g., "CLIPTextEncode",
    /// "KSampler").
    pub node_type: String,
    /// A list of node identifiers that were successfully executed before the
    /// error occurred.
    pub executed: Vec<String>,
    /// The error message from the exception, describing what went wrong.
    pub exception_message: String,
    /// The type of the exception that was raised (e.g., "ValueError",
    /// "RuntimeError").
    pub exception_type: String,
    /// A traceback of the error as a list of strings, showing the execution
    /// path that led to the error.
    pub traceback: Vec<String>,
    /// The current input values at the time of the error, mapping input names
    /// to their values.
    pub current_inputs: HashMap<String, Value>,
    /// The current output values at the time of the error, mapping output names
    /// to their values.
    pub current_outputs: HashMap<String, Value>,
}

/// Event payload indicating that the execution result was obtained from the
/// cache rather than recalculated.
///
/// This structure is received when ComfyUI uses cached results for nodes in the
/// workflow, which can significantly speed up execution when identical
/// operations are performed.
#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionCachedEventData {
    /// A list of node identifiers that were retrieved from the cache instead of
    /// being re-executed.
    pub nodes: Vec<String>,
    /// The prompt ID associated with the cached execution, linking this event
    /// to a specific workflow run.
    pub prompt_id: String,
    /// Unix timestamp indicating when the cached execution result was
    /// retrieved, useful for timing analysis.
    pub timestamp: u64,
}

/// Event payload for an interrupted execution, containing details about the
/// interruption.
///
/// This structure is received when the workflow execution is manually
/// interrupted or terminated before completion, providing context about what
/// was executing at the time of interruption.
#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionInterruptedEventData {
    /// The prompt ID associated with the interruption, identifying the workflow
    /// execution that was stopped.
    pub prompt_id: String,
    /// The identifier of the node where the execution was interrupted,
    /// indicating which operation was in progress.
    pub node_id: String,
    /// The type of the node that was interrupted (e.g., "KSampler",
    /// "VAEDecode"), helping identify what operation was stopped.
    pub node_type: String,
    /// A list of node identifiers that were successfully executed before the
    /// interruption occurred.
    pub executed: Vec<String>,
}

/// Event payload indicating successful completion of workflow execution.
///
/// This structure is received when an entire workflow has completed execution
/// successfully. It serves as a final notification that all nodes in the
/// workflow have been processed without errors, and the workflow is complete.
#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionSuccessEventData {
    /// The prompt ID associated with the successful execution, identifying the
    /// completed workflow.
    pub prompt_id: String,
}

/// `Prompt` param for
/// [`ComfyUIClient::post_prompt`](crate::ComfyUIClient::post_prompt).
///
/// Represents a prompt that can be submitted to the ComfyUI server for
/// execution. The prompt defines the workflow to be executed, including all
/// nodes and their connections, as well as the input parameters for each node.
pub enum Prompt<'a> {
    /// A string slice representing the prompt in JSON format.
    ///
    /// Use this variant when you have a JSON string representation of the
    /// workflow.
    Str(&'a str),

    /// A JSON value representing the prompt data.
    ///
    /// Use this variant when you have already parsed the workflow into a
    /// serde_json Value.
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
        let ev = ComfyEvent::Status {
            data: StatusEventData {
                status: StatusEventStatus {
                    exec_info: ExecInfo { queue_remaining: 0 },
                },
            },
            sid: None,
        };
        let value = serde_json::to_value(&ev).unwrap();
        assert_eq!(
            value,
            json!({
                "type": "status",
                "data": {
                    "status": {
                        "exec_info": {
                            "queue_remaining": 0,
                        }
                    }
                },
                "sid": null
            })
        );

        let ev = ComfyEvent::ExecutionStart {
            data: ExecutionStartEventData {
                prompt_id: "xxxxxx".to_string(),
                timestamp: 123456789,
            },
        };
        let value = serde_json::to_value(&ev).unwrap();
        assert_eq!(
            value,
            json!({
                "type": "execution_start",
                "data": {
                    "prompt_id": "xxxxxx",
                    "timestamp": 123456789
                }
            })
        );
    }
}
