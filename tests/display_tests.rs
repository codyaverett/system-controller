use system_controller::server::display::*;
use system_controller::platform::traits::*;
use system_controller::platform::MockPlatform;
use mockall::predicate::*;
use anyhow::Result;
use std::time::Duration;

#[cfg(test)]
mod screen_capture_tests {
    use super::*;

    #[tokio::test]
    async fn test_screen_capture_single_display() {
        let mut mock_platform = MockPlatform::new();
        let expected_data = vec![0x89, 0x50, 0x4E, 0x47]; // PNG header
        
        mock_platform.expect_capture_screen()
            .with(eq(0))
            .times(1)
            .returning(move |_| Ok(expected_data.clone()));

        let display_controller = DisplayController::new(Box::new(mock_platform));
        let result = display_controller.capture_screen(0).await;
        
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.len(), 4);
        assert_eq!(data[0..4], vec![0x89, 0x50, 0x4E, 0x47]);
    }

    #[tokio::test]
    async fn test_screen_capture_invalid_display() {
        let mock_platform = MockPlatform::new();
        let display_controller = DisplayController::new(Box::new(mock_platform));
        
        let result = display_controller.capture_screen(999).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid display"));
    }

    #[tokio::test]
    async fn test_screen_capture_with_compression() {
        let mut mock_platform = MockPlatform::new();
        let large_data = vec![0u8; 1024]; // Simulate large screen data
        
        mock_platform.expect_capture_screen()
            .with(eq(0))
            .times(1)
            .returning(move |_| Ok(large_data.clone()));

        let display_controller = DisplayController::with_compression(
            Box::new(mock_platform), 
            CompressionType::PNG
        );
        let result = display_controller.capture_screen(0).await;
        
        assert!(result.is_ok());
        let compressed_data = result.unwrap();
        assert!(!compressed_data.is_empty());
    }

    #[tokio::test]
    async fn test_screen_capture_streaming() {
        let mock_platform = MockPlatform::new();
        
        let display_controller = DisplayController::new(Box::new(mock_platform));
        let mut stream = display_controller.create_capture_stream(0, Duration::from_millis(10)).await;
        
        // Get first 3 frames - these are mock frames from the stream itself
        let frame1 = stream.next_frame().await;
        let frame2 = stream.next_frame().await;
        let frame3 = stream.next_frame().await;
        
        assert!(frame1.is_ok());
        assert!(frame2.is_ok());
        assert!(frame3.is_ok());
        assert_eq!(frame1.unwrap(), vec![0x89, 0x50, 0x4E, 0x47]);
    }

    #[tokio::test]
    async fn test_screen_capture_differential() {
        let mut mock_platform = MockPlatform::new();
        let frame1 = vec![1, 2, 3, 4];
        let frame2 = vec![1, 2, 5, 6]; // Changed last 2 bytes
        
        mock_platform.expect_capture_screen()
            .times(2)
            .returning({
                let mut call_count = 0;
                move |_| {
                    call_count += 1;
                    if call_count == 1 {
                        Ok(frame1.clone())
                    } else {
                        Ok(frame2.clone())
                    }
                }
            });

        let mut display_controller = DisplayController::with_differential_capture(Box::new(mock_platform));
        
        let result1 = display_controller.capture_screen_differential(0).await;
        let result2 = display_controller.capture_screen_differential(0).await;
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        
        let diff = result2.unwrap();
        assert!(diff.has_changes);
        assert_eq!(diff.changed_regions.len(), 1);
    }
}

#[cfg(test)]
mod display_enumeration_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_displays() {
        let mut mock_platform = MockPlatform::new();
        let expected_displays = vec![
            DisplayInfo {
                id: 0,
                name: "Primary Display".to_string(),
                width: 1920,
                height: 1080,
                x: 0,
                y: 0,
                is_primary: true,
            },
            DisplayInfo {
                id: 1,
                name: "Secondary Display".to_string(),
                width: 1440,
                height: 900,
                x: 1920,
                y: 0,
                is_primary: false,
            },
        ];

        mock_platform.expect_get_displays()
            .times(1)
            .returning(move || Ok(expected_displays.clone()));

        let display_controller = DisplayController::new(Box::new(mock_platform));
        let result = display_controller.get_displays().await;
        
        assert!(result.is_ok());
        let displays = result.unwrap();
        assert_eq!(displays.len(), 2);
        assert!(displays[0].is_primary);
        assert!(!displays[1].is_primary);
    }

    #[tokio::test]
    async fn test_get_primary_display() {
        let mut mock_platform = MockPlatform::new();
        let displays = vec![
            DisplayInfo {
                id: 1,
                name: "Secondary".to_string(),
                width: 1440,
                height: 900,
                x: 1920,
                y: 0,
                is_primary: false,
            },
            DisplayInfo {
                id: 0,
                name: "Primary".to_string(),
                width: 1920,
                height: 1080,
                x: 0,
                y: 0,
                is_primary: true,
            },
        ];

        mock_platform.expect_get_displays()
            .times(1)
            .returning(move || Ok(displays.clone()));

        let display_controller = DisplayController::new(Box::new(mock_platform));
        let result = display_controller.get_primary_display().await;
        
        assert!(result.is_ok());
        let primary = result.unwrap();
        assert!(primary.is_some());
        assert_eq!(primary.unwrap().id, 0);
    }

    #[tokio::test]
    async fn test_display_metrics() {
        let mut mock_platform = MockPlatform::new();
        let displays = vec![
            DisplayInfo {
                id: 0,
                name: "4K Display".to_string(),
                width: 3840,
                height: 2160,
                x: 0,
                y: 0,
                is_primary: true,
            },
        ];

        mock_platform.expect_get_displays()
            .times(1)
            .returning(move || Ok(displays.clone()));

        let display_controller = DisplayController::new(Box::new(mock_platform));
        let metrics = display_controller.get_display_metrics().await;
        
        assert!(metrics.is_ok());
        let metrics = metrics.unwrap();
        assert_eq!(metrics.total_width, 3840);
        assert_eq!(metrics.total_height, 2160);
        assert_eq!(metrics.display_count, 1);
    }
}

#[cfg(test)]
mod window_management_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_window_at_position() {
        let mut mock_platform = MockPlatform::new();
        let expected_window = WindowInfo {
            id: 12345,
            title: "Test Window".to_string(),
            x: 100,
            y: 100,
            width: 800,
            height: 600,
            process_name: "test.exe".to_string(),
        };

        mock_platform.expect_get_window_at_position()
            .with(eq(150), eq(200))
            .times(1)
            .returning(move |_, _| Ok(Some(expected_window.clone())));

        let display_controller = DisplayController::new(Box::new(mock_platform));
        let result = display_controller.get_window_at_position(150, 200).await;
        
        assert!(result.is_ok());
        let window = result.unwrap();
        assert!(window.is_some());
        assert_eq!(window.unwrap().title, "Test Window");
    }

    #[tokio::test]
    async fn test_get_window_at_empty_position() {
        let mut mock_platform = MockPlatform::new();
        
        mock_platform.expect_get_window_at_position()
            .with(eq(50), eq(50))
            .times(1)
            .returning(|_, _| Ok(None));

        let display_controller = DisplayController::new(Box::new(mock_platform));
        let result = display_controller.get_window_at_position(50, 50).await;
        
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_list_all_windows() {
        let mut mock_platform = MockPlatform::new();
        let expected_windows = vec![
            WindowInfo {
                id: 1,
                title: "Browser".to_string(),
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
                process_name: "browser.exe".to_string(),
            },
            WindowInfo {
                id: 2,
                title: "Editor".to_string(),
                x: 100,
                y: 100,
                width: 800,
                height: 600,
                process_name: "editor.exe".to_string(),
            },
        ];

        mock_platform.expect_list_windows()
            .times(1)
            .returning(move || Ok(expected_windows.clone()));

        let display_controller = DisplayController::new(Box::new(mock_platform));
        let result = display_controller.list_windows().await;
        
        assert!(result.is_ok());
        let windows = result.unwrap();
        assert_eq!(windows.len(), 2);
        assert_eq!(windows[0].title, "Browser");
        assert_eq!(windows[1].title, "Editor");
    }

    #[tokio::test]
    async fn test_get_active_window() {
        let mut mock_platform = MockPlatform::new();
        let active_window = WindowInfo {
            id: 999,
            title: "Active Window".to_string(),
            x: 200,
            y: 200,
            width: 1000,
            height: 700,
            process_name: "active.exe".to_string(),
        };

        mock_platform.expect_get_active_window()
            .times(1)
            .returning(move || Ok(Some(active_window.clone())));

        let display_controller = DisplayController::new(Box::new(mock_platform));
        let result = display_controller.get_active_window().await;
        
        assert!(result.is_ok());
        let window = result.unwrap();
        assert!(window.is_some());
        assert_eq!(window.unwrap().title, "Active Window");
    }

    #[tokio::test]
    async fn test_window_filtering() {
        let mut mock_platform = MockPlatform::new();
        let windows = vec![
            WindowInfo {
                id: 1,
                title: "Important Document.pdf".to_string(),
                x: 0,
                y: 0,
                width: 800,
                height: 600,
                process_name: "pdf_viewer.exe".to_string(),
            },
            WindowInfo {
                id: 2,
                title: "System Tray".to_string(),
                x: 1800,
                y: 1040,
                width: 120,
                height: 40,
                process_name: "system.exe".to_string(),
            },
        ];

        mock_platform.expect_list_windows()
            .times(1)
            .returning(move || Ok(windows.clone()));

        let display_controller = DisplayController::new(Box::new(mock_platform));
        let filter = WindowFilter {
            min_width: Some(200),
            min_height: Some(200),
            exclude_system_windows: true,
        };
        let result = display_controller.list_windows_filtered(filter).await;
        
        assert!(result.is_ok());
        let filtered_windows = result.unwrap();
        assert_eq!(filtered_windows.len(), 1);
        assert_eq!(filtered_windows[0].title, "Important Document.pdf");
    }
}

#[cfg(test)]
mod compression_tests {
    use super::*;

    #[test]
    fn test_png_compression() {
        let compressor = ImageCompressor::new(CompressionType::PNG);
        let raw_data = vec![255u8; 1000]; // White pixels
        
        let result = compressor.compress(&raw_data, 100, 100);
        assert!(result.is_ok());
        
        let compressed = result.unwrap();
        assert!(compressed.len() < raw_data.len()); // Should be smaller
        assert!(compressed.starts_with(&[0x89, 0x50, 0x4E, 0x47])); // PNG signature
    }

    #[test]
    fn test_jpeg_compression() {
        let compressor = ImageCompressor::new(CompressionType::JPEG);
        let raw_data = vec![128u8; 1000]; // Gray pixels
        
        let result = compressor.compress(&raw_data, 100, 100);
        assert!(result.is_ok());
        
        let compressed = result.unwrap();
        assert!(compressed.len() < raw_data.len());
        assert!(compressed.starts_with(&[0xFF, 0xD8])); // JPEG signature
    }

    #[test]
    fn test_compression_quality() {
        let compressor_high = ImageCompressor::with_quality(CompressionType::JPEG, 95);
        let compressor_low = ImageCompressor::with_quality(CompressionType::JPEG, 50);
        let raw_data = vec![200u8; 10000];
        
        let high_quality = compressor_high.compress(&raw_data, 100, 100).unwrap();
        let low_quality = compressor_low.compress(&raw_data, 100, 100).unwrap();
        
        // Higher quality should result in larger file size
        assert!(high_quality.len() > low_quality.len());
    }
}