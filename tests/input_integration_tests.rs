use system_controller::server::input::*;
use system_controller::platform::traits::*;
use system_controller::platform::MockPlatform;
use mockall::predicate::*;
use std::time::Duration;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_mouse_control_integration() {
        let mut mock_platform = MockPlatform::new();
        
        // Set up expectations for a complete mouse interaction sequence
        mock_platform.expect_mouse_move()
            .with(eq(100), eq(200))
            .times(1)
            .returning(|_, _| Ok(()));
            
        mock_platform.expect_mouse_click()
            .with(eq(MouseButton::Left), eq(100), eq(200))
            .times(1)
            .returning(|_, _, _| Ok(()));
            
        mock_platform.expect_mouse_scroll()
            .with(eq(0), eq(-3))
            .times(1)
            .returning(|_, _| Ok(()));

        let input_controller = InputController::new(Box::new(mock_platform));
        
        // Execute a complete mouse interaction sequence
        assert!(input_controller.move_mouse(100, 200).await.is_ok());
        assert!(input_controller.click_mouse(MouseButton::Left, 100, 200).await.is_ok());
        assert!(input_controller.scroll_mouse(0, -3).await.is_ok());
    }

    #[tokio::test]
    async fn test_keyboard_control_integration() {
        let mut mock_platform = MockPlatform::new();
        
        // Set up expectations for keyboard interaction sequence
        mock_platform.expect_key_press()
            .with(eq("Control".to_string()))
            .times(1)
            .returning(|_| Ok(()));
            
        mock_platform.expect_key_press()
            .with(eq("a".to_string()))
            .times(1)
            .returning(|_| Ok(()));
            
        mock_platform.expect_key_release()
            .with(eq("a".to_string()))
            .times(1)
            .returning(|_| Ok(()));
            
        mock_platform.expect_key_release()
            .with(eq("Control".to_string()))
            .times(1)
            .returning(|_| Ok(()));
            
        mock_platform.expect_type_text()
            .with(eq("Hello World".to_string()))
            .times(1)
            .returning(|_| Ok(()));

        let input_controller = InputController::new(Box::new(mock_platform));
        
        // Execute keyboard interaction sequence
        assert!(input_controller.key_combination(&["Control", "a"]).await.is_ok());
        assert!(input_controller.type_text("Hello World").await.is_ok());
    }

    #[tokio::test]
    async fn test_error_handling_propagation() {
        let mut mock_platform = MockPlatform::new();
        
        // Platform returns an error
        mock_platform.expect_mouse_move()
            .with(eq(100), eq(200))
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("Platform error")));

        let input_controller = InputController::new(Box::new(mock_platform));
        
        let result = input_controller.move_mouse(100, 200).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Platform error"));
    }

    #[tokio::test]
    async fn test_concurrent_input_operations() {
        let mut mock_platform = MockPlatform::new();
        
        // Allow multiple concurrent operations
        mock_platform.expect_mouse_move()
            .times(3)
            .returning(|_, _| Ok(()));

        let input_controller = std::sync::Arc::new(
            InputController::new(Box::new(mock_platform))
        );
        
        // Execute concurrent operations
        let controller1 = input_controller.clone();
        let controller2 = input_controller.clone();
        let controller3 = input_controller.clone();
        
        let (result1, result2, result3) = tokio::join!(
            controller1.move_mouse(100, 100),
            controller2.move_mouse(200, 200),
            controller3.move_mouse(300, 300)
        );
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());
    }
}