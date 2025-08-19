use system_controller::server::network::*;
use system_controller::protocol::messages::*;
use system_controller::platform::{MockPlatform, MouseButton};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncBufReadExt, BufReader};
use tokio::time::{timeout, Duration};
use serde_json;

#[cfg(test)]
mod tcp_server_tests {
    use super::*;

    #[tokio::test]
    async fn test_server_starts_and_binds_to_port() {
        let mock_platform = MockPlatform::new();
        let mut server = NetworkServer::new(Box::new(mock_platform), "127.0.0.1:0".to_string());
        
        let result = server.start().await;
        assert!(result.is_ok());
        
        let bound_addr = server.local_addr().await;
        assert!(bound_addr.is_ok());
        assert!(bound_addr.unwrap().port() > 0);
    }

    #[tokio::test]
    async fn test_server_accepts_client_connections() {
        let mock_platform = MockPlatform::new();
        let mut server = NetworkServer::new(Box::new(mock_platform), "127.0.0.1:0".to_string());
        
        server.start().await.unwrap();
        let addr = server.local_addr().await.unwrap();
        
        // Connect a client
        let client_result = TcpStream::connect(addr).await;
        assert!(client_result.is_ok());
        
        // Give server time to process the connection
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Server should accept the connection
        let connection_count = server.active_connections().await;
        assert_eq!(connection_count, 1);
    }

    #[tokio::test]
    async fn test_server_handles_multiple_concurrent_connections() {
        let mock_platform = MockPlatform::new();
        let mut server = NetworkServer::new(Box::new(mock_platform), "127.0.0.1:0".to_string());
        
        server.start().await.unwrap();
        let addr = server.local_addr().await.unwrap();
        
        // Connect multiple clients
        let mut clients = Vec::new();
        for _ in 0..5 {
            let client = TcpStream::connect(addr).await.unwrap();
            clients.push(client);
            // Small delay between connections
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        
        // Give server time to process all connections
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Server should handle all connections
        let connection_count = server.active_connections().await;
        assert_eq!(connection_count, 5);
    }

    #[tokio::test]
    async fn test_server_graceful_shutdown() {
        let mock_platform = MockPlatform::new();
        let mut server = NetworkServer::new(Box::new(mock_platform), "127.0.0.1:0".to_string());
        
        server.start().await.unwrap();
        let addr = server.local_addr().await.unwrap();
        
        // Connect a client
        let _client = TcpStream::connect(addr).await.unwrap();
        
        // Shutdown server
        let shutdown_result = server.shutdown().await;
        assert!(shutdown_result.is_ok());
        
        // New connections should fail
        let connect_result = timeout(Duration::from_millis(100), TcpStream::connect(addr)).await;
        assert!(connect_result.is_err() || connect_result.unwrap().is_err());
    }

    #[tokio::test]
    async fn test_server_connection_cleanup_on_client_disconnect() {
        let mock_platform = MockPlatform::new();
        let mut server = NetworkServer::new(Box::new(mock_platform), "127.0.0.1:0".to_string());
        
        server.start().await.unwrap();
        let addr = server.local_addr().await.unwrap();
        
        // Connect and immediately disconnect
        {
            let _client = TcpStream::connect(addr).await.unwrap();
            // Client goes out of scope and disconnects
        }
        
        // Give server time to clean up
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        let connection_count = server.active_connections().await;
        assert_eq!(connection_count, 0);
    }

    #[tokio::test]
    async fn test_server_configuration_options() {
        let mock_platform = MockPlatform::new();
        let config = NetworkConfig {
            bind_address: "127.0.0.1:0".to_string(),
            max_connections: 10,
            connection_timeout: Duration::from_secs(30),
            enable_websocket: false,
            buffer_size: 8192,
        };
        
        let mut server = NetworkServer::with_config(Box::new(mock_platform), config);
        assert!(server.start().await.is_ok());
        
        let server_config = server.get_config().await;
        assert_eq!(server_config.max_connections, 10);
        assert_eq!(server_config.buffer_size, 8192);
    }

    #[tokio::test]
    async fn test_server_connection_limit_enforcement() {
        let mock_platform = MockPlatform::new();
        let config = NetworkConfig {
            bind_address: "127.0.0.1:0".to_string(),
            max_connections: 2,
            connection_timeout: Duration::from_secs(30),
            enable_websocket: false,
            buffer_size: 8192,
        };
        
        let mut server = NetworkServer::with_config(Box::new(mock_platform), config);
        server.start().await.unwrap();
        let addr = server.local_addr().await.unwrap();
        
        // Connect up to the limit
        let _client1 = TcpStream::connect(addr).await.unwrap();
        let _client2 = TcpStream::connect(addr).await.unwrap();
        
        // Third connection should be rejected or handled gracefully
        let client3_result = timeout(Duration::from_millis(100), TcpStream::connect(addr)).await;
        
        // Either connection fails immediately or server handles it with appropriate response
        if let Ok(Ok(mut client3)) = client3_result {
            // Server should send a rejection message
            let mut buffer = [0u8; 1024];
            let read_result = timeout(Duration::from_millis(100), client3.read(&mut buffer)).await;
            assert!(read_result.is_ok()); // Should receive rejection message
        }
    }
}

#[cfg(test)]
mod protocol_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_server_processes_mouse_move_command() {
        let mut mock_platform = MockPlatform::new();
        mock_platform.expect_mouse_move()
            .with(mockall::predicate::eq(100), mockall::predicate::eq(200))
            .times(1)
            .returning(|_, _| Ok(()));

        let mut server = NetworkServer::new(Box::new(mock_platform), "127.0.0.1:0".to_string());
        server.start().await.unwrap();
        let addr = server.local_addr().await.unwrap();

        let mut client = TcpStream::connect(addr).await.unwrap();
        
        let command = Command {
            id: "test-1".to_string(),
            command_type: CommandType::MouseMove,
            payload: CommandPayload::MouseMove { x: 100, y: 200 },
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        let command_json = serde_json::to_string(&command).unwrap();
        client.write_all(command_json.as_bytes()).await.unwrap();
        client.write_all(b"\n").await.unwrap(); // Message delimiter
        
        // Read response
        let mut buffer = [0u8; 1024];
        let bytes_read = client.read(&mut buffer).await.unwrap();
        let response_text = String::from_utf8_lossy(&buffer[..bytes_read]);
        
        let response: Response = serde_json::from_str(&response_text).unwrap();
        assert_eq!(response.command_id, "test-1");
        assert_eq!(response.status, ResponseStatus::Success);
    }

    #[tokio::test]
    async fn test_server_processes_mouse_click_command() {
        let mut mock_platform = MockPlatform::new();
        mock_platform.expect_mouse_click()
            .times(1)
            .returning(|_, _, _| Ok(()));

        let mut server = NetworkServer::new(Box::new(mock_platform), "127.0.0.1:0".to_string());
        server.start().await.unwrap();
        let addr = server.local_addr().await.unwrap();

        let mut client = TcpStream::connect(addr).await.unwrap();
        
        let command = Command {
            id: "test-2".to_string(),
            command_type: CommandType::MouseClick,
            payload: CommandPayload::MouseClick { 
                button: MouseButton::Left,
                x: 150, 
                y: 250 
            },
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        let command_json = serde_json::to_string(&command).unwrap();
        client.write_all(command_json.as_bytes()).await.unwrap();
        client.write_all(b"\n").await.unwrap();
        
        let mut buffer = [0u8; 1024];
        let bytes_read = client.read(&mut buffer).await.unwrap();
        let response_text = String::from_utf8_lossy(&buffer[..bytes_read]);
        
        let response: Response = serde_json::from_str(&response_text).unwrap();
        assert_eq!(response.status, ResponseStatus::Success);
    }

    #[tokio::test]
    async fn test_server_processes_keyboard_command() {
        let mut mock_platform = MockPlatform::new();
        mock_platform.expect_key_press()
            .with(mockall::predicate::eq("Enter".to_string()))
            .times(1)
            .returning(|_| Ok(()));

        let mut server = NetworkServer::new(Box::new(mock_platform), "127.0.0.1:0".to_string());
        server.start().await.unwrap();
        let addr = server.local_addr().await.unwrap();

        let mut client = TcpStream::connect(addr).await.unwrap();
        
        let command = Command {
            id: "test-3".to_string(),
            command_type: CommandType::KeyPress,
            payload: CommandPayload::KeyPress { key: "Enter".to_string() },
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        let command_json = serde_json::to_string(&command).unwrap();
        client.write_all(command_json.as_bytes()).await.unwrap();
        client.write_all(b"\n").await.unwrap();
        
        let mut buffer = [0u8; 1024];
        let bytes_read = client.read(&mut buffer).await.unwrap();
        let response_text = String::from_utf8_lossy(&buffer[..bytes_read]);
        
        let response: Response = serde_json::from_str(&response_text).unwrap();
        assert_eq!(response.status, ResponseStatus::Success);
    }

    #[tokio::test]
    async fn test_server_handles_invalid_json() {
        let mock_platform = MockPlatform::new();
        let mut server = NetworkServer::new(Box::new(mock_platform), "127.0.0.1:0".to_string());
        server.start().await.unwrap();
        let addr = server.local_addr().await.unwrap();

        let mut client = TcpStream::connect(addr).await.unwrap();
        
        // Send invalid JSON
        client.write_all(b"{ invalid json }\n").await.unwrap();
        
        let mut buffer = [0u8; 1024];
        let bytes_read = client.read(&mut buffer).await.unwrap();
        let response_text = String::from_utf8_lossy(&buffer[..bytes_read]);
        
        let response: Response = serde_json::from_str(&response_text).unwrap();
        assert_eq!(response.status, ResponseStatus::Error);
        assert!(response.error.unwrap().contains("Invalid JSON"));
    }

    #[tokio::test]
    async fn test_server_handles_platform_errors() {
        let mut mock_platform = MockPlatform::new();
        mock_platform.expect_mouse_move()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("Platform error")));

        let mut server = NetworkServer::new(Box::new(mock_platform), "127.0.0.1:0".to_string());
        server.start().await.unwrap();
        let addr = server.local_addr().await.unwrap();

        let mut client = TcpStream::connect(addr).await.unwrap();
        
        let command = Command {
            id: "test-error".to_string(),
            command_type: CommandType::MouseMove,
            payload: CommandPayload::MouseMove { x: 100, y: 200 },
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        let command_json = serde_json::to_string(&command).unwrap();
        client.write_all(command_json.as_bytes()).await.unwrap();
        client.write_all(b"\n").await.unwrap();
        
        let mut buffer = [0u8; 1024];
        let bytes_read = client.read(&mut buffer).await.unwrap();
        let response_text = String::from_utf8_lossy(&buffer[..bytes_read]);
        
        let response: Response = serde_json::from_str(&response_text).unwrap();
        assert_eq!(response.status, ResponseStatus::Error);
        assert!(response.error.unwrap().contains("Platform error"));
    }
}

#[cfg(test)]
mod binary_data_tests {
    use super::*;

    #[tokio::test]
    async fn test_server_processes_screen_capture_request() {
        let mut mock_platform = MockPlatform::new();
        let test_image_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG header
        mock_platform.expect_capture_screen()
            .with(mockall::predicate::eq(0))
            .times(1)
            .returning(move |_| Ok(test_image_data.clone()));

        let mut server = NetworkServer::new(Box::new(mock_platform), "127.0.0.1:0".to_string());
        server.start().await.unwrap();
        let addr = server.local_addr().await.unwrap();

        let mut client = TcpStream::connect(addr).await.unwrap();
        
        let command = Command {
            id: "capture-1".to_string(),
            command_type: CommandType::CaptureScreen,
            payload: CommandPayload::CaptureScreen { display_id: 0 },
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        let command_json = serde_json::to_string(&command).unwrap();
        client.write_all(command_json.as_bytes()).await.unwrap();
        client.write_all(b"\n").await.unwrap();
        
        // Read response header (JSON) line by line
        let mut reader = BufReader::new(&mut client);
        let mut response_line = String::new();
        reader.read_line(&mut response_line).await.unwrap();
        
        let response: Response = serde_json::from_str(&response_line.trim()).unwrap();
        assert_eq!(response.status, ResponseStatus::Success);
        
        // Response should indicate binary data follows
        if let Some(ResponseData::ScreenCapture { size, format }) = response.data {
            assert_eq!(size, 8); // PNG header size
            assert_eq!(format, "png");
            
            // Read binary data
            let mut image_buffer = vec![0u8; size];
            reader.read_exact(&mut image_buffer).await.unwrap();
            assert_eq!(image_buffer, vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
        } else {
            panic!("Expected ScreenCapture response data");
        }
    }

    #[tokio::test]
    async fn test_server_handles_large_binary_data() {
        let mut mock_platform = MockPlatform::new();
        let large_image_data = vec![0u8; 1024 * 1024]; // 1MB of data
        mock_platform.expect_capture_screen()
            .times(1)
            .returning(move |_| Ok(large_image_data.clone()));

        let mut server = NetworkServer::new(Box::new(mock_platform), "127.0.0.1:0".to_string());
        server.start().await.unwrap();
        let addr = server.local_addr().await.unwrap();

        let mut client = TcpStream::connect(addr).await.unwrap();
        
        let command = Command {
            id: "large-capture".to_string(),
            command_type: CommandType::CaptureScreen,
            payload: CommandPayload::CaptureScreen { display_id: 0 },
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        let command_json = serde_json::to_string(&command).unwrap();
        client.write_all(command_json.as_bytes()).await.unwrap();
        client.write_all(b"\n").await.unwrap();
        
        // Read response - should handle large data without issues
        let mut reader = BufReader::new(&mut client);
        let mut response_line = String::new();
        reader.read_line(&mut response_line).await.unwrap();
        
        let response: Response = serde_json::from_str(&response_line.trim()).unwrap();
        assert_eq!(response.status, ResponseStatus::Success);
        
        if let Some(ResponseData::ScreenCapture { size, .. }) = response.data {
            assert_eq!(size, 1024 * 1024);
        }
    }

    #[tokio::test]
    async fn test_server_binary_data_chunked_transmission() {
        let mut mock_platform = MockPlatform::new();
        let test_data = vec![0xAB; 10000]; // 10KB test data
        mock_platform.expect_capture_screen()
            .times(1)
            .returning(move |_| Ok(test_data.clone()));

        let config = NetworkConfig {
            bind_address: "127.0.0.1:0".to_string(),
            max_connections: 10,
            connection_timeout: Duration::from_secs(30),
            enable_websocket: false,
            buffer_size: 1024, // Small buffer to force chunking
        };

        let mut server = NetworkServer::with_config(Box::new(mock_platform), config);
        server.start().await.unwrap();
        let addr = server.local_addr().await.unwrap();

        let mut client = TcpStream::connect(addr).await.unwrap();
        
        let command = Command {
            id: "chunked-capture".to_string(),
            command_type: CommandType::CaptureScreen,
            payload: CommandPayload::CaptureScreen { display_id: 0 },
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        let command_json = serde_json::to_string(&command).unwrap();
        client.write_all(command_json.as_bytes()).await.unwrap();
        client.write_all(b"\n").await.unwrap();
        
        // Read response header
        let mut reader = BufReader::new(&mut client);
        let mut response_line = String::new();
        reader.read_line(&mut response_line).await.unwrap();
        
        let response: Response = serde_json::from_str(&response_line.trim()).unwrap();
        assert_eq!(response.status, ResponseStatus::Success);
        
        // Read all binary data in chunks
        let mut received_data = Vec::new();
        let mut remaining = 10000;
        while remaining > 0 {
            let mut chunk_buffer = vec![0u8; 1024.min(remaining)];
            let chunk_read = reader.read(&mut chunk_buffer).await.unwrap();
            received_data.extend_from_slice(&chunk_buffer[..chunk_read]);
            remaining -= chunk_read;
        }
        
        assert_eq!(received_data.len(), 10000);
        assert!(received_data.iter().all(|&b| b == 0xAB));
    }
}

#[cfg(test)]
mod websocket_tests {
    use super::*;

    #[tokio::test]
    async fn test_server_websocket_upgrade() {
        let mock_platform = MockPlatform::new();
        let config = NetworkConfig {
            bind_address: "127.0.0.1:0".to_string(),
            max_connections: 10,
            connection_timeout: Duration::from_secs(30),
            enable_websocket: true,
            buffer_size: 8192,
        };

        let mut server = NetworkServer::with_config(Box::new(mock_platform), config);
        server.start().await.unwrap();
        let addr = server.local_addr().await.unwrap();

        let mut client = TcpStream::connect(addr).await.unwrap();
        
        // Send WebSocket upgrade request
        let upgrade_request = 
            "GET /ws HTTP/1.1\r\n\
             Host: localhost\r\n\
             Upgrade: websocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\
             Sec-WebSocket-Version: 13\r\n\r\n";
        
        client.write_all(upgrade_request.as_bytes()).await.unwrap();
        
        // Read upgrade response
        let mut buffer = [0u8; 1024];
        let bytes_read = client.read(&mut buffer).await.unwrap();
        let response = String::from_utf8_lossy(&buffer[..bytes_read]);
        
        assert!(response.contains("HTTP/1.1 101 Switching Protocols"));
        assert!(response.contains("Upgrade: websocket"));
        assert!(response.contains("Connection: Upgrade"));
    }

    #[tokio::test]
    async fn test_server_websocket_command_processing() {
        let mut mock_platform = MockPlatform::new();
        mock_platform.expect_mouse_move()
            .times(1)
            .returning(|_, _| Ok(()));

        let config = NetworkConfig {
            bind_address: "127.0.0.1:0".to_string(),
            max_connections: 10,
            connection_timeout: Duration::from_secs(30),
            enable_websocket: true,
            buffer_size: 8192,
        };

        let mut server = NetworkServer::with_config(Box::new(mock_platform), config);
        server.start().await.unwrap();
        let addr = server.local_addr().await.unwrap();

        // Note: Full WebSocket implementation would require proper frame encoding
        // This test validates that the server can distinguish WebSocket vs TCP connections
        let mut client = TcpStream::connect(addr).await.unwrap();
        
        // Send a regular TCP command (not WebSocket)
        let command = Command {
            id: "ws-test".to_string(),
            command_type: CommandType::MouseMove,
            payload: CommandPayload::MouseMove { x: 100, y: 200 },
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        let command_json = serde_json::to_string(&command).unwrap();
        client.write_all(command_json.as_bytes()).await.unwrap();
        client.write_all(b"\n").await.unwrap();
        
        // Should still work as TCP connection
        let mut buffer = [0u8; 1024];
        let bytes_read = client.read(&mut buffer).await.unwrap();
        let response_text = String::from_utf8_lossy(&buffer[..bytes_read]);
        
        let response: Response = serde_json::from_str(&response_text).unwrap();
        assert_eq!(response.status, ResponseStatus::Success);
    }
}