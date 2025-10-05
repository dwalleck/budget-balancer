// Main integration test runner
// This file is the entry point for cargo test --test integration_tests

// Set up test environment before any modules are loaded
// This sets the CSV rate limiter to 50ms for fast test execution
#[ctor::ctor]
fn init() {
    std::env::set_var("CSV_RATE_LIMIT_MS", "50");
}

mod integration;
