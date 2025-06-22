use serde::{Deserialize, Serialize};

/// Shared interface traits and types for the MCP server and Tauri plugin
/// This ensures both sides maintain compatible function signatures

// Window manager operation parameters
#[derive(Debug, Serialize, Deserialize)]
pub struct WindowManagerParams {
    pub window_label: Option<String>,
    pub operation: String,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

// Window manager operation result
#[derive(Debug, Serialize, Deserialize)]
pub struct WindowManagerResult {
    pub success: bool,
    pub error: Option<String>,
}

// Text input parameters
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextInputParams {
    pub text: String,
    pub delay_ms: Option<u64>,
    pub initial_delay_ms: Option<u64>,
}

// Text input result
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextInputResult {
    pub success: bool,
    pub chars_typed: u32,
    pub duration_ms: u64,
    pub error: Option<String>,
}

// Mouse movement parameters
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MouseMovementParams {
    pub x: i32,
    pub y: i32,
    pub relative: Option<bool>,
    pub click: Option<bool>,
    pub button: Option<String>, // "left", "right", or "middle"
}

// Mouse movement result
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MouseMovementResult {
    pub success: bool,
    pub duration_ms: u64,
    pub position: Option<(i32, i32)>,
    pub error: Option<String>,
}

/// Main interface trait for MCP functionality
pub trait McpInterface {
    /// Manages window operations (resize, position, show/hide, etc.)
    fn manage_window_shared(
        &self,
        params: WindowManagerParams,
    ) -> std::result::Result<WindowManagerResult, String>;

    /// Simulates keyboard text input
    fn simulate_text_input_shared(
        &self,
        params: TextInputParams,
    ) -> std::result::Result<TextInputResult, String>;

    /// Simulates mouse movement
    fn simulate_mouse_movement_shared(
        &self,
        params: MouseMovementParams,
    ) -> std::result::Result<MouseMovementResult, String>;

    // Add other shared functions here
}

/// Command string constants for socket commands
pub mod commands {
    pub const PING: &str = "ping";
    pub const GET_DOM: &str = "get_dom";
    pub const MANAGE_LOCAL_STORAGE: &str = "manage_local_storage";
    pub const EXECUTE_JS: &str = "execute_js";
    pub const MANAGE_WINDOW: &str = "manage_window";
    pub const SIMULATE_TEXT_INPUT: &str = "simulate_text_input";
    pub const SIMULATE_MOUSE_MOVEMENT: &str = "simulate_mouse_movement";
    pub const GET_ELEMENT_POSITION: &str = "get_element_position";
    pub const SEND_TEXT_TO_ELEMENT: &str = "send_text_to_element";
}
