use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    #[serde(rename = "type")]
    pub command_type: CommandType,
    pub payload: CommandPayload,
    pub timestamp: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandType {
    MouseMove,
    MouseClick,
    MouseScroll,
    KeyPress,
    KeyRelease,
    TypeText,
    CaptureScreen,
    GetDisplays,
    GetWindowInfo,
    ListWindows,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CommandPayload {
    #[serde(rename = "mouse_move")]
    MouseMove { x: i32, y: i32 },
    #[serde(rename = "mouse_click")]
    MouseClick { button: crate::platform::traits::MouseButton, x: i32, y: i32 },
    #[serde(rename = "mouse_scroll")]
    MouseScroll { x: i32, y: i32 },
    #[serde(rename = "key_press")]
    KeyPress { key: String },
    #[serde(rename = "key_release")]
    KeyRelease { key: String },
    #[serde(rename = "type_text")]
    TypeText { text: String },
    #[serde(rename = "capture_screen")]
    CaptureScreen { display_id: u32 },
    #[serde(rename = "get_displays")]
    GetDisplays {},
    #[serde(rename = "get_window_info")]
    GetWindowInfo { x: i32, y: i32 },
    #[serde(rename = "list_windows")]
    ListWindows {},
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response {
    pub command_id: String,
    pub status: ResponseStatus,
    pub error: Option<String>,
    pub data: Option<ResponseData>,
    pub timestamp: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Success,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponseData {
    #[serde(rename = "screen_capture")]
    ScreenCapture {
        size: usize,
        format: String,
    },
    #[serde(rename = "display_info")]
    DisplayInfo {
        displays: Vec<DisplayData>,
    },
    #[serde(rename = "window_info")]
    WindowInfo {
        windows: Vec<WindowData>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplayData {
    pub id: u32,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub is_primary: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowData {
    pub id: u64,
    pub title: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub process_name: String,
}

impl Command {
    pub fn validate(&self) -> Result<()> {
        match &self.payload {
            CommandPayload::MouseMove { x, y } => {
                if *x < 0 || *y < 0 {
                    return Err(anyhow!("Mouse coordinates must be non-negative"));
                }
            }
            CommandPayload::MouseClick { x, y, .. } => {
                if *x < 0 || *y < 0 {
                    return Err(anyhow!("Mouse coordinates must be non-negative"));
                }
            }
            CommandPayload::KeyPress { key } | CommandPayload::KeyRelease { key } => {
                if key.is_empty() {
                    return Err(anyhow!("Key cannot be empty"));
                }
            }
            CommandPayload::TypeText { text } => {
                if text.len() > 1000 {
                    return Err(anyhow!("Text input too long"));
                }
            }
            _ => {}
        }
        Ok(())
    }
}