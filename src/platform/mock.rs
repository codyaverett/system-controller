use mockall::mock;
use super::traits::*;
use anyhow::Result;

mock! {
    pub Platform {}

    impl PlatformController for Platform {
        fn mouse_move(&self, x: i32, y: i32) -> Result<()>;
        fn mouse_click(&self, button: MouseButton, x: i32, y: i32) -> Result<()>;
        fn mouse_scroll(&self, x: i32, y: i32) -> Result<()>;
        fn key_press(&self, key: String) -> Result<()>;
        fn key_release(&self, key: String) -> Result<()>;
        fn type_text(&self, text: String) -> Result<()>;
        fn capture_screen(&self, display_id: u32) -> Result<Vec<u8>>;
        fn get_displays(&self) -> Result<Vec<DisplayInfo>>;
        fn get_window_at_position(&self, x: i32, y: i32) -> Result<Option<WindowInfo>>;
        fn list_windows(&self) -> Result<Vec<WindowInfo>>;
        fn get_active_window(&self) -> Result<Option<WindowInfo>>;
    }
}