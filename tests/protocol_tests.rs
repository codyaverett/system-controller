use serde_json;
use system_controller::protocol::messages::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_serialization_roundtrip() {
        let cmd = Command {
            id: "test-001".to_string(),
            command_type: CommandType::MouseMove,
            payload: CommandPayload::MouseMove { x: 100, y: 200 },
            timestamp: "2025-08-18T10:30:00Z".to_string(),
        };

        let json = serde_json::to_string(&cmd).expect("Failed to serialize");
        let deserialized: Command = serde_json::from_str(&json).expect("Failed to deserialize");
        
        assert_eq!(cmd.id, deserialized.id);
        assert_eq!(cmd.command_type, deserialized.command_type);
        assert_eq!(cmd.timestamp, deserialized.timestamp);
    }

    #[test]
    fn test_mouse_move_command_validation() {
        let cmd = Command {
            id: "test-002".to_string(),
            command_type: CommandType::MouseMove,
            payload: CommandPayload::MouseMove { x: 100, y: 200 },
            timestamp: "2025-08-18T10:30:00Z".to_string(),
        };

        assert!(cmd.validate().is_ok());
    }

    #[test]
    fn test_invalid_coordinates_rejected() {
        let cmd = Command {
            id: "test-003".to_string(),
            command_type: CommandType::MouseMove,
            payload: CommandPayload::MouseMove { x: -1, y: -1 },
            timestamp: "2025-08-18T10:30:00Z".to_string(),
        };

        assert!(cmd.validate().is_err());
    }

    #[test]
    fn test_keyboard_command_validation() {
        let cmd = Command {
            id: "test-004".to_string(),
            command_type: CommandType::KeyPress,
            payload: CommandPayload::KeyPress { key: "Enter".to_string() },
            timestamp: "2025-08-18T10:30:00Z".to_string(),
        };

        assert!(cmd.validate().is_ok());
    }

    #[test]
    fn test_response_error_format() {
        let error_resp = Response {
            command_id: "test-005".to_string(),
            status: ResponseStatus::Error,
            error: Some("Command validation failed".to_string()),
            data: None,
            timestamp: "2025-08-18T10:30:00Z".to_string(),
        };

        let json = serde_json::to_string(&error_resp).expect("Failed to serialize error");
        assert!(json.contains("\"error\""));
        assert!(json.contains("Command validation failed"));
    }
}