//! Integration tests for the error system.

#[cfg(test)]
mod tests {
    use crate::error::*;
    use std::path::PathBuf;

    /// Test the complete error system workflow
    #[test]
    fn test_complete_error_workflow() {
        // Initialize error system
        integration::initialize_error_system().expect("Failed to initialize error system");
        
        // Create a parse error
        let location = SourceLocation::new(5, 17, 100)
            .with_file(PathBuf::from("test.v"))
            .with_length(1);
            
        let error = integration::parse_error(
            ErrorCode::EXPECTED_TOKEN,
            "expected `;`, found `}`".to_string(),
            Some(location),
            Some(vec![";".to_string()]),
            Some("}".to_string()),
        );
        
        // Test formatting
        let system = integration::get_error_system();
        let formatted = integration::format_error(&system, &error);
        assert!(!formatted.is_empty());
        assert!(formatted.contains("error"));
        assert!(formatted.contains("E0001"));

        // Test suggestions
        let suggestions = integration::generate_suggestions(&system, &error);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.message.to_lowercase().contains("semicolon")));
    }
    
    /// Test different formatter types
    #[test]
    fn test_formatter_switching() {
        integration::initialize_error_system().expect("Failed to initialize");
        
        let error = integration::type_error(
            ErrorCode::TYPE_MISMATCH,
            "type mismatch".to_string(),
            Some(SourceLocation::new(3, 10, 50)),
            Some("u64".to_string()),
            Some("u32".to_string()),
        );
        
        // Test terminal formatter
        {
            let mut sys = integration::get_error_system_mut();
            integration::set_formatter(&mut sys, "terminal").expect("Failed to set terminal formatter");
        }
        let terminal_output = {
            let sys = integration::get_error_system();
            integration::format_error(&sys, &error)
        };
        assert!(!terminal_output.is_empty());

        // Test JSON formatter
        {
            let mut sys = integration::get_error_system_mut();
            integration::set_formatter(&mut sys, "json").expect("Failed to set JSON formatter");
        }
        let json_output = {
            let sys = integration::get_error_system();
            integration::format_error(&sys, &error)
        };
        assert!(!json_output.is_empty());
        assert!(json_output.starts_with('{'));

        // Test LSP formatter
        {
            let mut sys = integration::get_error_system_mut();
            integration::set_formatter(&mut sys, "lsp").expect("Failed to set LSP formatter");
        }
        let lsp_output = {
            let sys = integration::get_error_system();
            integration::format_error(&sys, &error)
        };
        assert!(!lsp_output.is_empty());
        assert!(lsp_output.starts_with('{'));
    }
    
    /// Test error collector functionality
    #[test]
    fn test_error_collector() {
        let mut collector = integration::ErrorCollector::new();
        
        // Add various errors
        let parse_error = integration::parse_error(
            ErrorCode::EXPECTED_TOKEN,
            "parse error".to_string(),
            None,
            None,
            None,
        );
        
        let type_error = integration::type_error(
            ErrorCode::TYPE_MISMATCH,
            "type error".to_string(),
            None,
            Some("u64".to_string()),
            Some("string".to_string()),
        );
        
        let warning = integration::warning(
            ErrorCode::UNUSED_VARIABLE,
            "unused variable".to_string(),
            None,
            Some("Consider removing this variable".to_string()),
        );
        
        collector.error(parse_error);
        collector.error(type_error);
        collector.error(warning);
        
        // Validate collector state
        assert_eq!(collector.error_count(), 2);
        assert_eq!(collector.warning_count(), 1);
        assert!(collector.has_errors());
        assert!(collector.has_warnings());
        
        integration::initialize_error_system().expect("Failed to initialize");
        let system = integration::get_error_system();
        // Test formatting all errors
        let formatted_all = collector.format_all(&system);
        assert!(!formatted_all.is_empty());
        assert!(formatted_all.contains("parse error"));
        assert!(formatted_all.contains("type error"));
        assert!(formatted_all.contains("unused variable"));
    }
    
    /// Test template system with different error types
    #[test]
    fn test_template_system() {
        integration::initialize_error_system().expect("Failed to initialize");
        
        // Test parse error with semicolon suggestion
        let parse_error = integration::parse_error(
            ErrorCode::EXPECTED_TOKEN,
            "expected token".to_string(),
            Some(SourceLocation::new(5, 10, 100)),
            Some(vec![";".to_string()]),
            Some("}".to_string()),
        );
        
        let system = integration::get_error_system();
        let suggestions = integration::generate_suggestions(&system, &parse_error);
        let semicolon_suggestion = suggestions.iter()
            .find(|s| s.message.to_lowercase().contains("semicolon"));
        assert!(semicolon_suggestion.is_some());
        
        // Test type error with conversion suggestion
        let type_error = integration::type_error(
            ErrorCode::TYPE_MISMATCH,
            "type mismatch".to_string(),
            None,
            Some("u64".to_string()),
            Some("u32".to_string()),
        );
        
        let type_suggestions = integration::generate_suggestions(&system, &type_error);
        let conversion_suggestion = type_suggestions.iter()
            .find(|s| s.message.contains("as u64") || s.message.contains(".into()"));
        assert!(conversion_suggestion.is_some());
    }
    
    /// Test configuration loading and error registry
    #[test]
    fn test_configuration_loading() {
        let test_config = r#"
        [errors.E9999]
        title = "Test custom error: {message}"
        category = "test"
        severity = "error"
        description = "A custom error for testing configuration loading"
        
        [[errors.E9999.suggestions]]
        text = "This is a custom suggestion"
        "#;
        
        let mut system = ErrorSystem::new();
        system.load_config(test_config).expect("Failed to load test config");
        
        // The registry should now contain the custom error
        assert!(system.has_registry());
    }
    
    /// Test source context extraction and formatting
    #[test]
    fn test_source_context() {
        let source_code = r#"script Test {
    balance: u64;
    
    init() {
        let x = invalid_syntax
        balance = 100;
    }
}"#;
        
        let location = SourceLocation::new(5, 17, 0)
            .with_length(14)
            .with_file(PathBuf::from("test.v"));
        
        let context_result = context::extract_error_context(source_code, &location);
        assert!(context_result.is_ok());
        
        let context = context_result.unwrap();
        assert!(context.source_line.is_some());
        assert!(context.source_snippet.is_some());
        
        let source_line = context.source_line.unwrap();
        assert!(source_line.contains("invalid_syntax"));
    }
    
    /// Test error code formatting and display
    #[test]
    fn test_error_code_formatting() {
        let codes = [
            (ErrorCode::EXPECTED_TOKEN, "E0001"),
            (ErrorCode::TYPE_MISMATCH, "E1000"),
            (ErrorCode::UNDEFINED_VARIABLE, "E2000"),
            (ErrorCode::STACK_OVERFLOW_CODEGEN, "E3000"),
            (ErrorCode::DIVISION_BY_ZERO, "E4001"),
            (ErrorCode::FILE_NOT_FOUND, "E5000"),
        ];
        
        for (code, expected) in codes {
            assert_eq!(format!("{}", code), expected);
        }
    }
    
    /// Test error severity and category handling
    #[test]
    fn test_error_severity_and_categories() {
        // Test different severities
        let error = ErrorBuilder::new(ErrorCode::EXPECTED_TOKEN, "test".to_string())
            .severity(ErrorSeverity::Warning)
            .build();
        assert!(error.is_warning());
        assert!(!error.is_error());
        
        let error = ErrorBuilder::new(ErrorCode::EXPECTED_TOKEN, "test".to_string())
            .severity(ErrorSeverity::Error)
            .build();
        assert!(error.is_error());
        assert!(!error.is_warning());
        
        // Test categories
        let categories = [
            ErrorCategory::Syntax,
            ErrorCategory::Type,
            ErrorCategory::Semantic,
            ErrorCategory::Codegen,
            ErrorCategory::IO,
        ];
        
        for category in categories {
            let formatted = format!("{}", category);
            assert!(!formatted.is_empty());
        }
    }
    
    /// Test complex error scenarios with multiple related errors
    #[test]
    fn test_complex_error_scenarios() {
        let main_error = ErrorBuilder::new(
            ErrorCode::UNDEFINED_VARIABLE,
            "cannot find value `undefined_var`".to_string(),
        )
        .category(ErrorCategory::Semantic)
        .location(SourceLocation::new(10, 5, 200))
        .context(
            ErrorContext::new()
                .with_identifier("undefined_var".to_string())
        )
        .related(RelatedError::note(
            "variable was used here but never declared".to_string()
        ))
        .related(RelatedError::help(
            "try declaring the variable with `let undefined_var: Type = value;`".to_string()
        ))
        .build();
        
        // Test formatting with related errors
        integration::initialize_error_system().expect("Failed to initialize");
        let system = integration::get_error_system();
        let formatted = integration::format_error(&system, &main_error);
        assert!(formatted.contains("undefined_var"));
        assert!(formatted.contains("E2000"));
        
        // Test suggestions
        let suggestions = integration::generate_suggestions(&system, &main_error);
        let declaration_suggestion = suggestions.iter()
            .find(|s| s.message.to_lowercase().contains("declare"));
        assert!(declaration_suggestion.is_some());
    }
    
    /// Test performance with large numbers of errors
    #[test]
    fn test_performance_with_many_errors() {
        integration::initialize_error_system().expect("Failed to initialize");
        let mut collector = integration::ErrorCollector::new();
        
        // Generate many errors
        for i in 0..100 {
            let error = integration::parse_error(
                ErrorCode::EXPECTED_TOKEN,
                format!("error number {}", i),
                Some(SourceLocation::new(i as u32, 1, i * 10)),
                None,
                None,
            );
            collector.error(error);
        }
        
        assert_eq!(collector.error_count(), 100);
        
        // Test formatting performance
        let start = std::time::Instant::now();
        let system = integration::get_error_system();
        let formatted = collector.format_all(&system);
        let duration = start.elapsed();
        
        assert!(!formatted.is_empty());
        assert!(duration.as_millis() < 1000); // Should format 100 errors in under 1 second
    }
}
