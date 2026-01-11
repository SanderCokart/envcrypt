// Main entry point for CLI integration tests
// All test modules are compiled into a single binary for fast execution

mod common;
mod cli_tests;

// Re-export all test modules so they're discovered by the test runner
// The actual tests are in tests/cli_tests/*.rs modules
