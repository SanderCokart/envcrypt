# Testing

This project uses a feature-based test structure with multiple integration test files.

## Running Tests

### Standard Cargo (Serial Execution)

```bash
cargo test
```

**Note:** Cargo runs integration test files **serially** (one after another), which can be slow.

### cargo-nextest (Parallel Execution) - Recommended

For much faster test execution, use `cargo-nextest` which runs integration test files **in parallel**:

```bash
# Install cargo-nextest (one-time setup)
cargo install cargo-nextest --locked

# Run tests in parallel
cargo nextest run
```

**Performance:** With 7 test files, `cargo nextest run` is typically **10-20x faster** than `cargo test` because it runs all test files concurrently instead of sequentially.

## Test Structure

Tests are organized by feature:

- `tests/encrypt_basic.rs` - Basic encrypt functionality
- `tests/decrypt_basic.rs` - Basic decrypt functionality  
- `tests/roundtrip.rs` - Encrypt/decrypt roundtrip tests
- `tests/path_handling.rs` - Custom path and --input flag tests
- `tests/key_handling.rs` - Key parsing tests (base64 prefix, whitespace)
- `tests/env_flag.rs` - --env flag tests
- `tests/error_cases.rs` - Error condition tests
- `tests/common/mod.rs` - Shared test utilities

## Configuration

Test parallelization is configured in `.config/nextest.toml`. By default, tests run with as many threads as there are logical CPUs.
