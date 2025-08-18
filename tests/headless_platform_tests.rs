use system_controller::platform::headless::*;
use system_controller::platform::traits::*;
use system_controller::platform::factory::*;

#[cfg(test)]
mod headless_tests {
    use super::*;

    #[tokio::test]
    async fn test_headless_platform_creation() {
        let platform = HeadlessPlatform::new();
        // Should create without error
        assert!(true);
    }

    #[tokio::test]
    async fn test_headless_mouse_operations() {
        let platform = HeadlessPlatform::new_silent();
        
        // All mouse operations should succeed in headless mode
        assert!(platform.mouse_move(100, 200).is_ok());
        assert!(platform.mouse_click(MouseButton::Left, 150, 250).is_ok());
        assert!(platform.mouse_scroll(0, -3).is_ok());
    }

    #[tokio::test]
    async fn test_headless_keyboard_operations() {
        let platform = HeadlessPlatform::new_silent();
        
        // All keyboard operations should succeed in headless mode
        assert!(platform.key_press("Enter".to_string()).is_ok());
        assert!(platform.key_release("Enter".to_string()).is_ok());
        assert!(platform.type_text("Hello World".to_string()).is_ok());
    }

    #[tokio::test]
    async fn test_headless_display_operations() {
        let platform = HeadlessPlatform::new_silent();
        
        // Display operations should return mock data
        let screen_data = platform.capture_screen(0);
        assert!(screen_data.is_ok());
        assert!(!screen_data.unwrap().is_empty());
        
        let displays = platform.get_displays();
        assert!(displays.is_ok());
        assert_eq!(displays.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_headless_window_operations() {
        let platform = HeadlessPlatform::new_silent();
        
        // Window operations should return mock data
        let window_at_pos = platform.get_window_at_position(100, 100);
        assert!(window_at_pos.is_ok());
        assert!(window_at_pos.unwrap().is_some());
        
        let windows = platform.list_windows();
        assert!(windows.is_ok());
        assert_eq!(windows.unwrap().len(), 2);
        
        let active_window = platform.get_active_window();
        assert!(active_window.is_ok());
        assert!(active_window.unwrap().is_some());
    }
}

#[cfg(test)]
mod factory_tests {
    use super::*;

    #[test]
    fn test_platform_factory_creation() {
        let platform = PlatformFactory::create_platform();
        assert!(platform.is_ok());
    }

    #[test]
    fn test_platform_factory_by_name() {
        // Test headless creation
        let headless = PlatformFactory::create_platform_by_name("headless");
        assert!(headless.is_ok());
        
        // Test silent headless creation
        let silent = PlatformFactory::create_platform_by_name("headless-silent");
        assert!(silent.is_ok());
        
        // Test invalid platform
        let invalid = PlatformFactory::create_platform_by_name("nonexistent");
        assert!(invalid.is_err());
    }

    #[test]
    fn test_platform_capabilities() {
        let caps = PlatformFactory::get_platform_capabilities();
        
        // Capabilities should be consistent
        if caps.has_gui {
            assert!(caps.can_control_mouse);
            assert!(caps.can_control_keyboard);
        } else {
            assert!(!caps.supports_real_input);
        }
    }

    #[tokio::test]
    async fn test_factory_created_platform_works() {
        let platform = PlatformFactory::create_platform().unwrap();
        
        // Basic operations should work regardless of platform type
        let result = platform.mouse_move(100, 100);
        assert!(result.is_ok());
        
        let result = platform.key_press("a".to_string());
        assert!(result.is_ok());
    }
}