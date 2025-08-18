use system_controller::server::input::*;
use system_controller::platform::traits::*;
use system_controller::platform::MockPlatform;
use mockall::predicate::*;
use anyhow::Result;
use std::time::{Duration, Instant};

#[cfg(test)]
mod mouse_tests {
    use super::*;

    #[tokio::test]
    async fn test_mouse_move_coordinates_validation() {
        let mut mock_platform = MockPlatform::new();
        mock_platform.expect_mouse_move()
            .with(eq(100), eq(200))
            .times(1)
            .returning(|_, _| Ok(()));

        let input_controller = InputController::new(Box::new(mock_platform));
        let result = input_controller.move_mouse(100, 200).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mouse_move_negative_coordinates_rejected() {
        let mock_platform = MockPlatform::new();
        let input_controller = InputController::new(Box::new(mock_platform));
        
        let result = input_controller.move_mouse(-1, -1).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("negative coordinates"));
    }

    #[tokio::test]
    async fn test_mouse_click_valid_button() {
        let mut mock_platform = MockPlatform::new();
        mock_platform.expect_mouse_click()
            .with(eq(MouseButton::Left), eq(150), eq(250))
            .times(1)
            .returning(|_, _, _| Ok(()));

        let input_controller = InputController::new(Box::new(mock_platform));
        let result = input_controller.click_mouse(MouseButton::Left, 150, 250).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mouse_click_invalid_coordinates() {
        let mock_platform = MockPlatform::new();
        let input_controller = InputController::new(Box::new(mock_platform));
        
        let result = input_controller.click_mouse(MouseButton::Left, -1, 250).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mouse_scroll_functionality() {
        let mut mock_platform = MockPlatform::new();
        mock_platform.expect_mouse_scroll()
            .with(eq(0), eq(-3))
            .times(1)
            .returning(|_, _| Ok(()));

        let input_controller = InputController::new(Box::new(mock_platform));
        let result = input_controller.scroll_mouse(0, -3).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mouse_double_click() {
        let mut mock_platform = MockPlatform::new();
        mock_platform.expect_mouse_click()
            .with(eq(MouseButton::Left), eq(100), eq(100))
            .times(2)
            .returning(|_, _, _| Ok(()));

        let input_controller = InputController::new(Box::new(mock_platform));
        let result = input_controller.double_click_mouse(MouseButton::Left, 100, 100).await;
        
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod keyboard_tests {
    use super::*;

    #[tokio::test]
    async fn test_key_press_valid_key() {
        let mut mock_platform = MockPlatform::new();
        mock_platform.expect_key_press()
            .with(eq("Enter".to_string()))
            .times(1)
            .returning(|_| Ok(()));

        let input_controller = InputController::new(Box::new(mock_platform));
        let result = input_controller.press_key("Enter").await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_key_press_empty_key_rejected() {
        let mock_platform = MockPlatform::new();
        let input_controller = InputController::new(Box::new(mock_platform));
        
        let result = input_controller.press_key("").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty key"));
    }

    #[tokio::test]
    async fn test_key_release_functionality() {
        let mut mock_platform = MockPlatform::new();
        mock_platform.expect_key_release()
            .with(eq("Shift".to_string()))
            .times(1)
            .returning(|_| Ok(()));

        let input_controller = InputController::new(Box::new(mock_platform));
        let result = input_controller.release_key("Shift").await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_type_text_functionality() {
        let mut mock_platform = MockPlatform::new();
        mock_platform.expect_type_text()
            .with(eq("Hello World".to_string()))
            .times(1)
            .returning(|_| Ok(()));

        let input_controller = InputController::new(Box::new(mock_platform));
        let result = input_controller.type_text("Hello World").await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_type_text_length_limit() {
        let mock_platform = MockPlatform::new();
        let input_controller = InputController::new(Box::new(mock_platform));
        
        let long_text = "a".repeat(1001); // Exceeds 1000 char limit
        let result = input_controller.type_text(&long_text).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("text too long"));
    }

    #[tokio::test]
    async fn test_key_combination() {
        let mut mock_platform = MockPlatform::new();
        mock_platform.expect_key_press()
            .with(eq("Control".to_string()))
            .times(1)
            .returning(|_| Ok(()));
        mock_platform.expect_key_press()
            .with(eq("c".to_string()))
            .times(1)
            .returning(|_| Ok(()));
        mock_platform.expect_key_release()
            .with(eq("c".to_string()))
            .times(1)
            .returning(|_| Ok(()));
        mock_platform.expect_key_release()
            .with(eq("Control".to_string()))
            .times(1)
            .returning(|_| Ok(()));

        let input_controller = InputController::new(Box::new(mock_platform));
        let result = input_controller.key_combination(&["Control", "c"]).await;
        
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod rate_limiting_tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiting_blocks_excessive_requests() {
        let mock_platform = MockPlatform::new();
        let input_controller = InputController::with_rate_limit(
            Box::new(mock_platform), 
            2, // Max 2 requests per second
            Duration::from_secs(1)
        );

        // First two requests should succeed
        assert!(input_controller.move_mouse(100, 100).await.is_ok());
        assert!(input_controller.move_mouse(200, 200).await.is_ok());

        // Third request should be rate limited
        let result = input_controller.move_mouse(300, 300).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("rate limit"));
    }

    #[tokio::test]
    async fn test_rate_limiting_resets_after_window() {
        let mock_platform = MockPlatform::new();
        let input_controller = InputController::with_rate_limit(
            Box::new(mock_platform), 
            1, // Max 1 request per 100ms
            Duration::from_millis(100)
        );

        // First request should succeed
        assert!(input_controller.move_mouse(100, 100).await.is_ok());

        // Wait for rate limit window to reset
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Next request should succeed
        assert!(input_controller.move_mouse(200, 200).await.is_ok());
    }
}

#[cfg(test)]
mod input_validation_tests {
    use super::*;

    #[test]
    fn test_coordinate_bounds_validation() {
        let validator = InputValidator::new();
        
        assert!(validator.validate_coordinates(0, 0).is_ok());
        assert!(validator.validate_coordinates(1920, 1080).is_ok());
        assert!(validator.validate_coordinates(-1, 0).is_err());
        assert!(validator.validate_coordinates(0, -1).is_err());
        assert!(validator.validate_coordinates(i32::MAX, i32::MAX).is_ok());
    }

    #[test]
    fn test_key_name_validation() {
        let validator = InputValidator::new();
        
        assert!(validator.validate_key_name("Enter").is_ok());
        assert!(validator.validate_key_name("a").is_ok());
        assert!(validator.validate_key_name("F1").is_ok());
        assert!(validator.validate_key_name("").is_err());
        assert!(validator.validate_key_name("Invalid\x00Key").is_err());
    }

    #[test]
    fn test_text_input_sanitization() {
        let validator = InputValidator::new();
        
        assert!(validator.validate_text_input("Hello World").is_ok());
        assert!(validator.validate_text_input("").is_ok());
        assert!(validator.validate_text_input(&"a".repeat(1000)).is_ok());
        assert!(validator.validate_text_input(&"a".repeat(1001)).is_err());
        
        // Test for potentially dangerous sequences
        assert!(validator.validate_text_input("rm -rf /").is_ok()); // Should be allowed in text
        assert!(validator.validate_text_input("Text with\nnewlines").is_ok());
    }
}