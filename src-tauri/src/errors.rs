// Error handling utilities for sanitizing error messages
// Week 2: Error message sanitization

use std::fmt::Display;

/// Sanitizes a database error by logging it internally and returning a generic message
pub fn sanitize_db_error<E: Display>(error: E, operation: &str) -> String {
    // Log the detailed error internally for debugging
    eprintln!("Database error during {}: {}", operation, error);

    // Return a safe, generic message to the user
    format!("Failed to {}", operation)
}

/// Sanitizes a general error by logging it internally and returning a generic message
pub fn sanitize_error<E: Display>(error: E, context: &str, user_message: &str) -> String {
    // Log the detailed error internally for debugging
    eprintln!("{}: {}", context, error);

    // Return a safe, generic message to the user
    user_message.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_db_error() {
        let result = sanitize_db_error("connection refused", "load data");
        assert_eq!(result, "Failed to load data");
        // In real usage, eprintln would log "Database error during load data: connection refused"
    }

    #[test]
    fn test_sanitize_error() {
        let result = sanitize_error(
            "file not found",
            "File operation error",
            "Unable to access file"
        );
        assert_eq!(result, "Unable to access file");
        // In real usage, eprintln would log "File operation error: file not found"
    }
}
