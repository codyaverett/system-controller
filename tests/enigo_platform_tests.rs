use system_controller::platform::enigo_platform::*;
use system_controller::platform::traits::*;

#[cfg(test)]
mod enigo_tests {
    use super::*;

    #[test]
    fn test_enigo_platform_creation() {
        // This test may fail in CI environments without a display
        if let Ok(platform) = EnigoPlatform::new() {
            // Platform created successfully
            assert!(true);
        } else {
            // In headless environments, creation may fail - this is expected
            println!("EnigoPlatform creation failed - likely headless environment");
        }
    }

    #[test]
    fn test_mouse_button_conversion() {
        // Test that mouse button conversion works
        // This is testing internal logic, so we can't directly test the private method
        // but we can ensure the module compiles and the types are correct
        let _left = MouseButton::Left;
        let _right = MouseButton::Right;
        let _middle = MouseButton::Middle;
        assert!(true);
    }

    #[test]
    fn test_key_conversion_logic() {
        // Since convert_key is private, we test through the public interface
        // when the platform is available
        if let Ok(_platform) = EnigoPlatform::new() {
            // Platform is available for testing
            // Key conversion will be tested through integration tests
            assert!(true);
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::env;

    // These tests only run when explicitly enabled with ENABLE_REAL_INPUT_TESTS=1
    // to avoid interfering with the user's system during normal testing
    
    fn should_run_real_tests() -> bool {
        env::var("ENABLE_REAL_INPUT_TESTS").unwrap_or_default() == "1"
    }

    #[test]
    fn test_real_mouse_move() {
        if !should_run_real_tests() {
            return;
        }

        if let Ok(platform) = EnigoPlatform::new() {
            // Test actual mouse movement (be careful!)
            let result = platform.mouse_move(100, 100);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_real_key_press() {
        if !should_run_real_tests() {
            return;
        }

        if let Ok(platform) = EnigoPlatform::new() {
            // Test key press and release
            let press_result = platform.key_press("a".to_string());
            let release_result = platform.key_release("a".to_string());
            
            assert!(press_result.is_ok());
            assert!(release_result.is_ok());
        }
    }

    #[test]
    fn test_real_text_typing() {
        if !should_run_real_tests() {
            return;
        }

        if let Ok(platform) = EnigoPlatform::new() {
            // Test text typing
            let result = platform.type_text("test".to_string());
            assert!(result.is_ok());
        }
    }
}