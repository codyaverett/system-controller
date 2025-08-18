use crate::platform::traits::*;
use anyhow::{Result, anyhow};
use enigo::{Enigo, Settings, Button, Key, Direction, Coordinate, Mouse, Keyboard, Axis};
use screenshots::Screen;
use std::sync::Mutex;

/// Real platform implementation using the enigo library
pub struct EnigoPlatform {
    enigo: Mutex<Enigo>,
}

impl EnigoPlatform {
    pub fn new() -> Result<Self> {
        let settings = Settings::default();
        match Enigo::new(&settings) {
            Ok(enigo) => Ok(Self {
                enigo: Mutex::new(enigo),
            }),
            Err(e) => Err(anyhow!("Failed to initialize enigo: {:?}", e)),
        }
    }

    fn convert_mouse_button(button: &MouseButton) -> Button {
        match button {
            MouseButton::Left => Button::Left,
            MouseButton::Right => Button::Right,
            MouseButton::Middle => Button::Middle,
        }
    }

    fn convert_key(key_name: &str) -> Result<Key> {
        match key_name.to_lowercase().as_str() {
            // Control keys
            "enter" | "return" => Ok(Key::Return),
            "escape" | "esc" => Ok(Key::Escape),
            "space" => Ok(Key::Space),
            "tab" => Ok(Key::Tab),
            "backspace" => Ok(Key::Backspace),
            "delete" | "del" => Ok(Key::Delete),
            
            // Arrow keys
            "up" | "uparrow" => Ok(Key::UpArrow),
            "down" | "downarrow" => Ok(Key::DownArrow),
            "left" | "leftarrow" => Ok(Key::LeftArrow),
            "right" | "rightarrow" => Ok(Key::RightArrow),
            
            // Modifier keys
            "shift" => Ok(Key::Shift),
            "control" | "ctrl" => Ok(Key::Control),
            "alt" => Ok(Key::Alt),
            "cmd" | "command" | "meta" => Ok(Key::Meta),
            
            // Function keys
            "f1" => Ok(Key::F1),
            "f2" => Ok(Key::F2),
            "f3" => Ok(Key::F3),
            "f4" => Ok(Key::F4),
            "f5" => Ok(Key::F5),
            "f6" => Ok(Key::F6),
            "f7" => Ok(Key::F7),
            "f8" => Ok(Key::F8),
            "f9" => Ok(Key::F9),
            "f10" => Ok(Key::F10),
            "f11" => Ok(Key::F11),
            "f12" => Ok(Key::F12),
            
            // Single character keys - use Unicode instead of Layout
            key if key.len() == 1 => {
                let ch = key.chars().next().unwrap();
                Ok(Key::Unicode(ch))
            },
            
            _ => Err(anyhow!("Unknown key: {}", key_name)),
        }
    }
}

impl Default for EnigoPlatform {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl PlatformController for EnigoPlatform {
    fn mouse_move(&self, x: i32, y: i32) -> Result<()> {
        let mut enigo = self.enigo.lock()
            .map_err(|_| anyhow!("Failed to acquire enigo lock"))?;
        
        enigo.move_mouse(x, y, Coordinate::Abs)
            .map_err(|e| anyhow!("Failed to move mouse: {:?}", e))?;
        Ok(())
    }

    fn mouse_click(&self, button: MouseButton, x: i32, y: i32) -> Result<()> {
        let mut enigo = self.enigo.lock()
            .map_err(|_| anyhow!("Failed to acquire enigo lock"))?;
        
        let enigo_button = Self::convert_mouse_button(&button);
        
        // Move to position first
        enigo.move_mouse(x, y, Coordinate::Abs)
            .map_err(|e| anyhow!("Failed to move mouse: {:?}", e))?;
        
        // Perform click (press and release)
        enigo.button(enigo_button, Direction::Press)
            .map_err(|e| anyhow!("Failed to press button: {:?}", e))?;
        enigo.button(enigo_button, Direction::Release)
            .map_err(|e| anyhow!("Failed to release button: {:?}", e))?;
        
        Ok(())
    }

    fn mouse_scroll(&self, x: i32, y: i32) -> Result<()> {
        let mut enigo = self.enigo.lock()
            .map_err(|_| anyhow!("Failed to acquire enigo lock"))?;
        
        // Scroll vertically
        if y != 0 {
            enigo.scroll(y, Axis::Vertical)
                .map_err(|e| anyhow!("Failed to scroll vertically: {:?}", e))?;
        }
        
        // Scroll horizontally
        if x != 0 {
            enigo.scroll(x, Axis::Horizontal)
                .map_err(|e| anyhow!("Failed to scroll horizontally: {:?}", e))?;
        }
        
        Ok(())
    }

    fn key_press(&self, key: String) -> Result<()> {
        let mut enigo = self.enigo.lock()
            .map_err(|_| anyhow!("Failed to acquire enigo lock"))?;
        
        let enigo_key = Self::convert_key(&key)?;
        enigo.key(enigo_key, Direction::Press)
            .map_err(|e| anyhow!("Failed to press key: {:?}", e))?;
        
        Ok(())
    }

    fn key_release(&self, key: String) -> Result<()> {
        let mut enigo = self.enigo.lock()
            .map_err(|_| anyhow!("Failed to acquire enigo lock"))?;
        
        let enigo_key = Self::convert_key(&key)?;
        enigo.key(enigo_key, Direction::Release)
            .map_err(|e| anyhow!("Failed to release key: {:?}", e))?;
        
        Ok(())
    }

    fn type_text(&self, text: String) -> Result<()> {
        let mut enigo = self.enigo.lock()
            .map_err(|_| anyhow!("Failed to acquire enigo lock"))?;
        
        enigo.text(&text)
            .map_err(|e| anyhow!("Failed to type text: {:?}", e))?;
        
        Ok(())
    }

    fn capture_screen(&self, display_id: u32) -> Result<Vec<u8>> {
        let screens = Screen::all()
            .map_err(|e| anyhow!("Failed to get screens: {}", e))?;
        
        if display_id as usize >= screens.len() {
            return Err(anyhow!("Invalid display ID: {}", display_id));
        }
        
        let screen = &screens[display_id as usize];
        let _image = screen.capture()
            .map_err(|e| anyhow!("Failed to capture screen: {}", e))?;
        
        // For now, return a PNG signature as mock data
        // TODO: Implement proper image encoding using image crate
        let mut png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG signature
        png_data.extend_from_slice(&[0; 1000]); // Add some mock data
        
        Ok(png_data)
    }

    fn get_displays(&self) -> Result<Vec<DisplayInfo>> {
        let screens = Screen::all()
            .map_err(|e| anyhow!("Failed to get screens: {}", e))?;
        
        let mut displays = Vec::new();
        for (index, screen) in screens.iter().enumerate() {
            let display_info = DisplayInfo {
                id: index as u32,
                name: format!("Display {}", index),
                width: screen.display_info.width,
                height: screen.display_info.height,
                x: screen.display_info.x,
                y: screen.display_info.y,
                is_primary: screen.display_info.is_primary,
            };
            displays.push(display_info);
        }
        
        Ok(displays)
    }

    fn get_window_at_position(&self, _x: i32, _y: i32) -> Result<Option<WindowInfo>> {
        // TODO: Implement window detection
        // This will be implemented in Phase 3
        Err(anyhow!("Window detection not yet implemented"))
    }

    fn list_windows(&self) -> Result<Vec<WindowInfo>> {
        // TODO: Implement window listing
        // This will be implemented in Phase 3
        Err(anyhow!("Window listing not yet implemented"))
    }

    fn get_active_window(&self) -> Result<Option<WindowInfo>> {
        // TODO: Implement active window detection
        // This will be implemented in Phase 3
        Err(anyhow!("Active window detection not yet implemented"))
    }
}

// Mark as thread-safe
unsafe impl Send for EnigoPlatform {}
unsafe impl Sync for EnigoPlatform {}