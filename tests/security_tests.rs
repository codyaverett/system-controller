use system_controller::security::auth::*;
use system_controller::security::encryption::*;
use system_controller::security::audit::*;
use system_controller::security::permissions::*;
use system_controller::protocol::messages::*;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::time::{Duration, SystemTime};
use anyhow::Result;

#[cfg(test)]
mod authentication_tests {
    use super::*;

    #[tokio::test]
    async fn test_token_generation() {
        let auth_manager = AuthManager::new();
        let token = auth_manager.generate_token("test_user", Duration::from_secs(3600)).await;
        
        assert!(token.is_ok());
        let token = token.unwrap();
        assert!(!token.is_empty());
        assert!(token.len() >= 32); // Minimum token length for security
    }

    #[tokio::test]
    async fn test_token_validation() {
        let auth_manager = AuthManager::new();
        let token = auth_manager.generate_token("test_user", Duration::from_secs(3600)).await.unwrap();
        
        let validation = auth_manager.validate_token(&token).await;
        assert!(validation.is_ok());
        
        let user_info = validation.unwrap();
        assert_eq!(user_info.username, "test_user");
        assert!(user_info.expires_at > SystemTime::now());
    }

    #[tokio::test]
    async fn test_token_expiration() {
        let auth_manager = AuthManager::new();
        let token = auth_manager.generate_token("test_user", Duration::from_millis(1)).await.unwrap();
        
        // Wait for token to expire
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        let validation = auth_manager.validate_token(&token).await;
        assert!(validation.is_err());
        assert!(validation.unwrap_err().to_string().contains("expired"));
    }

    #[tokio::test]
    async fn test_invalid_token_rejection() {
        let auth_manager = AuthManager::new();
        
        let validation = auth_manager.validate_token("invalid_token_12345").await;
        assert!(validation.is_err());
        assert!(validation.unwrap_err().to_string().contains("invalid"));
    }

    #[tokio::test]
    async fn test_user_authentication_with_credentials() {
        let mut auth_manager = AuthManager::new();
        
        // Add test user
        auth_manager.add_user("testuser", "securepassword123", vec!["input_control".to_string()]).await.unwrap();
        
        let auth_result = auth_manager.authenticate_user("testuser", "securepassword123").await;
        assert!(auth_result.is_ok());
        
        let token = auth_result.unwrap();
        assert!(!token.is_empty());
    }

    #[tokio::test]
    async fn test_authentication_fails_with_wrong_password() {
        let mut auth_manager = AuthManager::new();
        
        auth_manager.add_user("testuser", "correctpassword", vec!["input_control".to_string()]).await.unwrap();
        
        let auth_result = auth_manager.authenticate_user("testuser", "wrongpassword").await;
        assert!(auth_result.is_err());
        assert!(auth_result.unwrap_err().to_string().contains("authentication failed"));
    }

    #[tokio::test]
    async fn test_authentication_fails_for_nonexistent_user() {
        let auth_manager = AuthManager::new();
        
        let auth_result = auth_manager.authenticate_user("nonexistent", "password").await;
        assert!(auth_result.is_err());
        assert!(auth_result.unwrap_err().to_string().contains("user not found"));
    }

    #[tokio::test]
    async fn test_token_refresh() {
        let auth_manager = AuthManager::new();
        let original_token = auth_manager.generate_token("test_user", Duration::from_secs(1)).await.unwrap();
        
        // Wait a bit then refresh
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        let new_token = auth_manager.refresh_token(&original_token, Duration::from_secs(3600)).await;
        assert!(new_token.is_ok());
        
        let new_token = new_token.unwrap();
        assert_ne!(original_token, new_token);
        
        // Original token should still be valid until it expires
        assert!(auth_manager.validate_token(&original_token).await.is_ok());
        assert!(auth_manager.validate_token(&new_token).await.is_ok());
    }

    #[tokio::test]
    async fn test_token_revocation() {
        let mut auth_manager = AuthManager::new();
        let token = auth_manager.generate_token("test_user", Duration::from_secs(3600)).await.unwrap();
        
        // Token should be valid initially
        assert!(auth_manager.validate_token(&token).await.is_ok());
        
        // Revoke the token
        auth_manager.revoke_token(&token).await.unwrap();
        
        // Token should now be invalid
        let validation = auth_manager.validate_token(&token).await;
        assert!(validation.is_err());
        assert!(validation.unwrap_err().to_string().contains("revoked"));
    }

    #[tokio::test]
    async fn test_password_hashing_security() {
        let mut auth_manager = AuthManager::new();
        
        auth_manager.add_user("user1", "password123", vec![]).await.unwrap();
        auth_manager.add_user("user2", "password123", vec![]).await.unwrap();
        
        // Passwords should be hashed differently (salted)
        let user1_hash = auth_manager.get_user_password_hash("user1").await.unwrap();
        let user2_hash = auth_manager.get_user_password_hash("user2").await.unwrap();
        
        assert_ne!(user1_hash, user2_hash); // Same password should have different hashes due to salt
        assert!(!user1_hash.contains("password123")); // Plain password should not appear in hash
    }
}

#[cfg(test)]
mod tls_encryption_tests {
    use super::*;

    #[tokio::test]
    async fn test_tls_server_creation() {
        use std::fs::File;
        use std::io::Write;
        
        // Create temporary test certificate files
        let mut cert_file = File::create("test_cert.pem").unwrap();
        cert_file.write_all(b"-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----").unwrap();
        
        let mut key_file = File::create("test_key.pem").unwrap();
        key_file.write_all(b"-----BEGIN PRIVATE KEY-----\ntest\n-----END PRIVATE KEY-----").unwrap();
        
        let tls_config = TlsConfig {
            cert_path: "test_cert.pem".to_string(),
            key_path: "test_key.pem".to_string(),
            ca_cert_path: None,
            require_client_cert: false,
        };
        
        let tls_manager = TlsManager::new(tls_config);
        assert!(tls_manager.is_ok());
        
        // Clean up test files
        std::fs::remove_file("test_cert.pem").ok();
        std::fs::remove_file("test_key.pem").ok();
    }

    #[tokio::test]
    async fn test_tls_certificate_generation() {
        let tls_manager = TlsManager::new_self_signed("localhost").await;
        assert!(tls_manager.is_ok());
        
        let tls_manager = tls_manager.unwrap();
        let cert_info = tls_manager.get_certificate_info().await;
        assert!(cert_info.is_ok());
        
        let cert_info = cert_info.unwrap();
        assert!(cert_info.subject.contains("localhost"));
        assert!(cert_info.expires_at > SystemTime::now());
    }

    #[tokio::test]
    async fn test_tls_handshake_simulation() {
        let tls_manager = TlsManager::new_self_signed("localhost").await.unwrap();
        
        // Simulate TLS handshake
        let handshake_result = tls_manager.perform_handshake_simulation().await;
        assert!(handshake_result.is_ok());
        
        let handshake_info = handshake_result.unwrap();
        assert!(!handshake_info.cipher_suite.is_empty());
        assert!(!handshake_info.protocol_version.is_empty());
    }

    #[tokio::test]
    async fn test_tls_data_encryption() {
        let tls_manager = TlsManager::new_self_signed("localhost").await.unwrap();
        let test_data = b"Hello, encrypted world!";
        
        let encrypted = tls_manager.encrypt_data(test_data).await;
        assert!(encrypted.is_ok());
        
        let encrypted_data = encrypted.unwrap();
        assert_ne!(test_data.to_vec(), encrypted_data);
        assert!(!encrypted_data.is_empty());
        
        let decrypted = tls_manager.decrypt_data(&encrypted_data).await;
        assert!(decrypted.is_ok());
        assert_eq!(test_data.to_vec(), decrypted.unwrap());
    }

    #[tokio::test]
    async fn test_client_certificate_validation() {
        use std::fs::File;
        use std::io::Write;
        
        // Create temporary test certificate files
        let mut cert_file = File::create("server_cert.pem").unwrap();
        cert_file.write_all(b"-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----").unwrap();
        
        let mut key_file = File::create("server_key.pem").unwrap();
        key_file.write_all(b"-----BEGIN PRIVATE KEY-----\ntest\n-----END PRIVATE KEY-----").unwrap();
        
        let mut ca_file = File::create("ca_cert.pem").unwrap();
        ca_file.write_all(b"-----BEGIN CERTIFICATE-----\nca\n-----END CERTIFICATE-----").unwrap();
        
        let tls_config = TlsConfig {
            cert_path: "server_cert.pem".to_string(),
            key_path: "server_key.pem".to_string(),
            ca_cert_path: Some("ca_cert.pem".to_string()),
            require_client_cert: true,
        };
        
        let tls_manager = TlsManager::new(tls_config).unwrap();
        
        // Test with valid client certificate
        let valid_cert = "valid_client_cert.pem";
        let validation = tls_manager.validate_client_certificate(valid_cert).await;
        assert!(validation.is_ok());
        
        // Test with invalid client certificate
        let invalid_cert = "invalid_client_cert.pem";
        let validation = tls_manager.validate_client_certificate(invalid_cert).await;
        assert!(validation.is_ok());
        assert_eq!(validation.unwrap(), false);
        
        // Clean up test files
        std::fs::remove_file("server_cert.pem").ok();
        std::fs::remove_file("server_key.pem").ok();
        std::fs::remove_file("ca_cert.pem").ok();
    }
}

#[cfg(test)]
mod authorization_tests {
    use super::*;

    #[tokio::test]
    async fn test_permission_system_creation() {
        let permission_manager = PermissionManager::new().await;
        assert!(permission_manager.is_ok());
    }

    #[tokio::test]
    async fn test_role_based_permissions() {
        let mut permission_manager = PermissionManager::new().await.unwrap();
        
        // Create roles
        permission_manager.create_role("admin", vec!["*".to_string()]).await.unwrap();
        permission_manager.create_role("operator", vec!["input_control".to_string(), "screen_capture".to_string()]).await.unwrap();
        permission_manager.create_role("viewer", vec!["screen_capture".to_string()]).await.unwrap();
        
        // Assign roles to users
        permission_manager.assign_role_to_user("admin", "admin").await.unwrap();
        permission_manager.assign_role_to_user("operator", "operator").await.unwrap();
        permission_manager.assign_role_to_user("viewer", "viewer").await.unwrap();
        
        // Test admin permissions
        assert!(permission_manager.check_permission("admin", "input_control").await.unwrap());
        assert!(permission_manager.check_permission("admin", "screen_capture").await.unwrap());
        assert!(permission_manager.check_permission("admin", "system_control").await.unwrap());
        
        // Test operator permissions
        assert!(permission_manager.check_permission("operator", "input_control").await.unwrap());
        assert!(permission_manager.check_permission("operator", "screen_capture").await.unwrap());
        assert!(!permission_manager.check_permission("operator", "system_control").await.unwrap());
        
        // Test viewer permissions
        assert!(!permission_manager.check_permission("viewer", "input_control").await.unwrap());
        assert!(permission_manager.check_permission("viewer", "screen_capture").await.unwrap());
    }

    #[tokio::test]
    async fn test_user_role_assignment() {
        let mut permission_manager = PermissionManager::new().await.unwrap();
        
        permission_manager.create_role("test_role", vec!["test_permission".to_string()]).await.unwrap();
        permission_manager.assign_role_to_user("test_user", "test_role").await.unwrap();
        
        let user_permissions = permission_manager.get_user_permissions("test_user").await.unwrap();
        assert!(user_permissions.contains(&"test_permission".to_string()));
    }

    #[tokio::test]
    async fn test_command_authorization() {
        let mut permission_manager = PermissionManager::new().await.unwrap();
        
        permission_manager.create_role("input_user", vec!["mouse_control".to_string(), "keyboard_control".to_string()]).await.unwrap();
        permission_manager.assign_role_to_user("test_user", "input_user").await.unwrap();
        
        // Test mouse command authorization
        let mouse_command = Command {
            id: "cmd1".to_string(),
            command_type: CommandType::MouseMove,
            payload: CommandPayload::MouseMove { x: 100, y: 200 },
            timestamp: "2025-08-19T10:00:00Z".to_string(),
        };
        
        let auth_result = permission_manager.authorize_command("test_user", &mouse_command).await;
        assert!(auth_result.is_ok());
        
        // Test unauthorized command
        let screen_command = Command {
            id: "cmd2".to_string(),
            command_type: CommandType::CaptureScreen,
            payload: CommandPayload::CaptureScreen { display_id: 0 },
            timestamp: "2025-08-19T10:00:00Z".to_string(),
        };
        
        let auth_result = permission_manager.authorize_command("test_user", &screen_command).await;
        assert!(auth_result.is_err());
        assert!(auth_result.unwrap_err().to_string().contains("permission denied"));
    }

    #[tokio::test]
    async fn test_time_based_access_control() {
        let mut permission_manager = PermissionManager::new().await.unwrap();
        
        let now = SystemTime::now();
        let one_hour_later = now + Duration::from_secs(3600);
        
        permission_manager.create_time_restricted_role(
            "time_limited",
            vec!["input_control".to_string()],
            now,
            one_hour_later
        ).await.unwrap();
        
        permission_manager.assign_role_to_user("time_user", "time_limited").await.unwrap();
        
        // Should have access now
        assert!(permission_manager.check_permission("time_user", "input_control").await.unwrap());
        
        // Create expired role
        let past_time = now - Duration::from_secs(3600);
        permission_manager.create_time_restricted_role(
            "expired_role",
            vec!["input_control".to_string()],
            past_time,
            now - Duration::from_secs(1800)
        ).await.unwrap();
        
        permission_manager.assign_role_to_user("expired_user", "expired_role").await.unwrap();
        
        // Should not have access (expired)
        assert!(!permission_manager.check_permission("expired_user", "input_control").await.unwrap());
    }
}

#[cfg(test)]
mod rate_limiting_tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_creation() {
        let rate_limiter = RateLimiter::new(10, Duration::from_secs(60));
        assert!(rate_limiter.is_ok());
    }

    #[tokio::test]
    async fn test_request_rate_limiting() {
        let mut rate_limiter = RateLimiter::new(3, Duration::from_secs(1)).unwrap();
        
        // First 3 requests should be allowed
        assert!(rate_limiter.check_rate_limit("user1").await.unwrap());
        assert!(rate_limiter.check_rate_limit("user1").await.unwrap());
        assert!(rate_limiter.check_rate_limit("user1").await.unwrap());
        
        // 4th request should be blocked
        assert!(!rate_limiter.check_rate_limit("user1").await.unwrap());
        
        // Different user should not be affected
        assert!(rate_limiter.check_rate_limit("user2").await.unwrap());
    }

    #[tokio::test]
    async fn test_rate_limit_window_reset() {
        let mut rate_limiter = RateLimiter::new(2, Duration::from_millis(100)).unwrap();
        
        // Use up the limit
        assert!(rate_limiter.check_rate_limit("user1").await.unwrap());
        assert!(rate_limiter.check_rate_limit("user1").await.unwrap());
        assert!(!rate_limiter.check_rate_limit("user1").await.unwrap());
        
        // Wait for window reset
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Should be allowed again
        assert!(rate_limiter.check_rate_limit("user1").await.unwrap());
    }

    #[tokio::test]
    async fn test_command_specific_rate_limiting() {
        let mut rate_limiter = CommandRateLimiter::new();
        
        // Configure different limits for different command types
        rate_limiter.set_command_limit(CommandType::MouseMove, 100, Duration::from_secs(1)).await.unwrap();
        rate_limiter.set_command_limit(CommandType::CaptureScreen, 1, Duration::from_secs(5)).await.unwrap();
        
        // Mouse moves should have high limit
        for _ in 0..50 {
            assert!(rate_limiter.check_command_rate("user1", CommandType::MouseMove).await.unwrap());
        }
        
        // Screen capture should have low limit
        assert!(rate_limiter.check_command_rate("user1", CommandType::CaptureScreen).await.unwrap());
        assert!(!rate_limiter.check_command_rate("user1", CommandType::CaptureScreen).await.unwrap());
    }

    #[tokio::test]
    async fn test_adaptive_rate_limiting() {
        let mut adaptive_limiter = AdaptiveRateLimiter::new().unwrap();
        
        // Simulate normal usage
        for _ in 0..10 {
            adaptive_limiter.record_request("normal_user", Duration::from_millis(100)).await.unwrap();
        }
        
        // Should maintain normal limits
        assert!(adaptive_limiter.check_rate_limit("normal_user").await.unwrap());
        
        // Simulate suspicious rapid requests
        for _ in 0..100 {
            adaptive_limiter.record_request("suspicious_user", Duration::from_millis(1)).await.unwrap();
        }
        
        // Should trigger stricter limits
        let limit_info = adaptive_limiter.get_current_limits("suspicious_user").await.unwrap();
        assert!(limit_info.is_restricted);
        assert!(limit_info.requests_per_minute < 60); // Should be reduced from default
    }
}

#[cfg(test)]
mod audit_logging_tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_logger_creation() {
        use uuid::Uuid;
        let unique_id = Uuid::new_v4().to_string();
        let audit_logger = AuditLogger::new(format!("test_audit_{}.log", unique_id)).await;
        assert!(audit_logger.is_ok());
    }

    #[tokio::test]
    async fn test_security_event_logging() {
        use uuid::Uuid;
        let unique_id = Uuid::new_v4().to_string();
        let mut audit_logger = AuditLogger::new(format!("test_security_audit_{}.log", unique_id)).await.unwrap();
        
        let security_event = SecurityEvent {
            event_type: SecurityEventType::AuthenticationSuccess,
            user_id: "test_user".to_string(),
            client_ip: "192.168.1.100".to_string(),
            timestamp: SystemTime::now(),
            details: "User successfully authenticated".to_string(),
            severity: SecuritySeverity::Info,
        };
        
        let result = audit_logger.log_security_event(security_event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_command_execution_logging() {
        use uuid::Uuid;
        let unique_id = Uuid::new_v4().to_string();
        let mut audit_logger = AuditLogger::new(format!("test_command_audit_{}.log", unique_id)).await.unwrap();
        
        let command = Command {
            id: "cmd123".to_string(),
            command_type: CommandType::MouseMove,
            payload: CommandPayload::MouseMove { x: 100, y: 200 },
            timestamp: "2025-08-19T10:00:00Z".to_string(),
        };
        
        let result = audit_logger.log_command_execution(
            "test_user",
            "192.168.1.100",
            &command,
            true // success
        ).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_failed_authentication_logging() {
        use uuid::Uuid;
        let unique_id = Uuid::new_v4().to_string();
        let mut audit_logger = AuditLogger::new(format!("test_auth_fail_audit_{}.log", unique_id)).await.unwrap();
        
        let result = audit_logger.log_authentication_failure(
            "invalid_user",
            "192.168.1.100",
            "Invalid credentials"
        ).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_audit_log_rotation() {
        use uuid::Uuid;
        let unique_id = Uuid::new_v4().to_string();
        let mut audit_logger = AuditLogger::with_rotation(
            format!("test_rotation_audit_{}.log", unique_id),
            1024, // 1KB max size
            5     // keep 5 files
        ).await.unwrap();
        
        // Generate enough logs to trigger rotation
        for i in 0..100 {
            let event = SecurityEvent {
                event_type: SecurityEventType::CommandExecution,
                user_id: format!("user_{}", i),
                client_ip: "192.168.1.100".to_string(),
                timestamp: SystemTime::now(),
                details: format!("Test log entry number {}", i),
                severity: SecuritySeverity::Info,
            };
            
            audit_logger.log_security_event(event).await.unwrap();
        }
        
        // Verify rotation occurred
        let rotation_info = audit_logger.get_rotation_info().await.unwrap();
        assert!(rotation_info.total_files > 1);
        assert!(rotation_info.current_file_size < 1024);
    }

    #[tokio::test]
    async fn test_audit_log_search() {
        use uuid::Uuid;
        let unique_id = Uuid::new_v4().to_string();
        let mut audit_logger = AuditLogger::new(format!("test_search_audit_{}.log", unique_id)).await.unwrap();
        
        // Log some test events
        for i in 0..10 {
            let event = SecurityEvent {
                event_type: if i % 2 == 0 { SecurityEventType::AuthenticationSuccess } else { SecurityEventType::AuthenticationFailure },
                user_id: format!("user_{}", i),
                client_ip: "192.168.1.100".to_string(),
                timestamp: SystemTime::now(),
                details: format!("Test event {}", i),
                severity: SecuritySeverity::Info,
            };
            
            audit_logger.log_security_event(event).await.unwrap();
        }
        
        // Search for authentication failures
        let search_results = audit_logger.search_events(
            SecurityEventType::AuthenticationFailure,
            SystemTime::now() - Duration::from_secs(60),
            SystemTime::now()
        ).await.unwrap();
        
        assert_eq!(search_results.len(), 5); // Should find 5 failure events
    }

    #[tokio::test]
    async fn test_real_time_monitoring() {
        use uuid::Uuid;
        let unique_id = Uuid::new_v4().to_string();
        let mut audit_logger = AuditLogger::new(format!("test_monitoring_audit_{}.log", unique_id)).await.unwrap();
        
        // Enable real-time monitoring
        let monitor = audit_logger.create_real_time_monitor().await.unwrap();
        
        // Log a high-severity event
        let critical_event = SecurityEvent {
            event_type: SecurityEventType::SecurityViolation,
            user_id: "attacker".to_string(),
            client_ip: "192.168.1.666".to_string(),
            timestamp: SystemTime::now(),
            details: "Potential security breach detected".to_string(),
            severity: SecuritySeverity::Critical,
        };
        
        audit_logger.log_security_event(critical_event).await.unwrap();
        
        // Monitor should detect the critical event
        let alerts = monitor.get_recent_alerts().await.unwrap();
        assert!(!alerts.is_empty());
        assert_eq!(alerts[0].severity, SecuritySeverity::Critical);
    }
}