use system_controller::platform::traits::*;
use system_controller::platform::mock::MockPlatform;
use mockall::predicate::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_mouse_move() {
        let mut mock = MockPlatform::new();
        mock.expect_mouse_move()
            .with(eq(100), eq(200))
            .times(1)
            .returning(|_, _| Ok(()));

        let result = mock.mouse_move(100, 200);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_mouse_click() {
        let mut mock = MockPlatform::new();
        mock.expect_mouse_click()
            .with(eq(MouseButton::Left), eq(150), eq(250))
            .times(1)
            .returning(|_, _, _| Ok(()));

        let result = mock.mouse_click(MouseButton::Left, 150, 250);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_key_press() {
        let mut mock = MockPlatform::new();
        mock.expect_key_press()
            .with(eq("Enter".to_string()))
            .times(1)
            .returning(|_| Ok(()));

        let result = mock.key_press("Enter".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_screen_capture() {
        let mut mock = MockPlatform::new();
        let expected_data = vec![1, 2, 3, 4]; // Mock image data
        
        mock.expect_capture_screen()
            .with(eq(0))
            .times(1)
            .returning(move |_| Ok(expected_data.clone()));

        let result = mock.capture_screen(0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_mock_get_displays() {
        let mut mock = MockPlatform::new();
        let expected_displays = vec![
            DisplayInfo {
                id: 0,
                name: "Primary".to_string(),
                width: 1920,
                height: 1080,
                x: 0,
                y: 0,
                is_primary: true,
            }
        ];

        mock.expect_get_displays()
            .times(1)
            .returning(move || Ok(expected_displays.clone()));

        let result = mock.get_displays();
        assert!(result.is_ok());
        let displays = result.unwrap();
        assert_eq!(displays.len(), 1);
        assert_eq!(displays[0].name, "Primary");
    }

    #[test]
    fn test_mock_get_window_info() {
        let mut mock = MockPlatform::new();
        let expected_window = WindowInfo {
            id: 12345,
            title: "Test Window".to_string(),
            x: 100,
            y: 100,
            width: 800,
            height: 600,
            process_name: "test.exe".to_string(),
        };

        mock.expect_get_window_at_position()
            .with(eq(100), eq(100))
            .times(1)
            .returning(move |_, _| Ok(Some(expected_window.clone())));

        let result = mock.get_window_at_position(100, 100);
        assert!(result.is_ok());
        let window = result.unwrap();
        assert!(window.is_some());
        assert_eq!(window.unwrap().title, "Test Window");
    }
}