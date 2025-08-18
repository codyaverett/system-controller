use system_controller::server::input::*;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_coordinate_validation_property(x in i32::MIN..i32::MAX, y in i32::MIN..i32::MAX) {
        let validator = InputValidator::new();
        let result = validator.validate_coordinates(x, y);
        
        // Property: coordinates should be valid if both are non-negative
        if x >= 0 && y >= 0 {
            prop_assert!(result.is_ok());
        } else {
            prop_assert!(result.is_err());
        }
    }

    #[test]
    fn test_text_length_validation_property(text in ".*") {
        let validator = InputValidator::new();
        let result = validator.validate_text_input(&text);
        
        // Property: text should be valid if length <= 1000
        if text.len() <= 1000 {
            prop_assert!(result.is_ok());
        } else {
            prop_assert!(result.is_err());
        }
    }

    #[test]
    fn test_key_name_validation_property(key in "[a-zA-Z0-9_]+") {
        let validator = InputValidator::new();
        let result = validator.validate_key_name(&key);
        
        // Property: non-empty alphanumeric keys should be valid
        if !key.is_empty() {
            prop_assert!(result.is_ok());
        }
    }

    #[test]
    fn test_coordinate_bounds_always_consistent(
        x1 in 0i32..1000,
        y1 in 0i32..1000,
        x2 in 0i32..1000,
        y2 in 0i32..1000
    ) {
        let validator = InputValidator::new();
        
        // Property: validation should be consistent for the same coordinates
        let result1 = validator.validate_coordinates(x1, y1);
        let result2 = validator.validate_coordinates(x1, y1);
        prop_assert_eq!(result1.is_ok(), result2.is_ok());
        
        // Property: valid coordinates should remain valid
        if result1.is_ok() {
            prop_assert!(validator.validate_coordinates(x2, y2).is_ok());
        }
    }
}