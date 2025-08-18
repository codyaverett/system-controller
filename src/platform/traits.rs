use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub id: u32,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub is_primary: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowInfo {
    pub id: u64,
    pub title: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub process_name: String,
}

pub trait PlatformController {
    // Mouse control
    fn mouse_move(&self, x: i32, y: i32) -> Result<()>;
    fn mouse_click(&self, button: MouseButton, x: i32, y: i32) -> Result<()>;
    fn mouse_scroll(&self, x: i32, y: i32) -> Result<()>;
    
    // Keyboard control
    fn key_press(&self, key: String) -> Result<()>;
    fn key_release(&self, key: String) -> Result<()>;
    fn type_text(&self, text: String) -> Result<()>;
    
    // Display management
    fn capture_screen(&self, display_id: u32) -> Result<Vec<u8>>;
    fn get_displays(&self) -> Result<Vec<DisplayInfo>>;
    
    // Window management
    fn get_window_at_position(&self, x: i32, y: i32) -> Result<Option<WindowInfo>>;
    fn list_windows(&self) -> Result<Vec<WindowInfo>>;
    fn get_active_window(&self) -> Result<Option<WindowInfo>>;
}