# envcrypt

A secure command-line tool for encrypting and decrypting environment files using AES-256-CBC encryption with HMAC-SHA256 authentication.

## Overview

`envcrypt` provides a simple and secure way to encrypt sensitive environment files (like `.env`) before committing them to version control. The encrypted files can be safely stored in repositories, and only users with the encryption key can decrypt them.

## Features

- **Strong Encryption**: AES-256-CBC encryption with HMAC-SHA256 authentication
- **Secure Key Derivation**: PBKDF2 with 100,000 iterations
- **Constant-Time Security**: Constant-time MAC verification to prevent timing attacks
- **Key Zeroization**: Automatic memory clearing of sensitive keys
- **Environment Support**: Support for multiple environments (local, production, etc.)
- **Flexible Paths**: Custom input/output paths
- **Feature Flags**: Modular compilation with optional features

## Installation

### From Source

1. Clone the repository:
```bash
git clone <repository-url>
cd envcrypt
```

2. Build and install:
```bash
./install.sh
```

The install script will:
- Build the release binary
- Install it to `~/.envcrypt/bin/envcrypt`
- Add it to your PATH (in your shell config file)

3. Reload your shell configuration:
```bash
source ~/.bashrc  # or ~/.zshrc, etc.
```

### Using Cargo

```bash
cargo install --path .
```

Or if you have the repository locally:
```bash
cargo build --release
# Binary will be at target/release/envcrypt
```

## Quick Start

### Encrypting a File

1. Create or edit your `.env` file:
```bash
echo "API_KEY=secret123" > .env
```

2. Encrypt it:
```bash
envcrypt encrypt
```

You'll be prompted to either:
- Generate a new encryption key (default)
- Use a custom key

3. Save the encryption key securely! You'll need it to decrypt later.

The encrypted file will be saved as `.env.encrypted`.

### Decrypting a File

```bash
envcrypt decrypt
```

You'll be prompted for the decryption key. The decrypted file will be saved as `.env`.

## Usage

### Basic Commands

#### Encrypt

```bash
envcrypt encrypt
```

Encrypts `.env` to `.env.encrypted` by default.

#### Decrypt

```bash
envcrypt decrypt
```

Decrypts `.env.encrypted` to `.env` by default.

### Command-Line Options

#### Encryption Options

```bash
envcrypt encrypt [OPTIONS]
```

- `--cipher <CIPHER>`: Cipher to use (default: `AES-256-CBC`)
- `--key <KEY>`: Encryption key (if not provided, will prompt)
- `--input <PATH>`: Input file path (default: `.env`, or `.env.{env}` if `--env` is specified)
- `--env <ENV>`: Environment name (e.g., `local`, `production`). When specified:
  - Default input: `.env.{env}`
  - Default output: `.env.{env}.encrypted`

#### Decryption Options

```bash
envcrypt decrypt [OPTIONS]
```

- `--cipher <CIPHER>`: Cipher to use (default: `AES-256-CBC`)
- `--key <KEY>`: Decryption key (if not provided, will prompt)
- `--input <PATH>`: Input encrypted file path (default: `.env.encrypted`)

### Examples

#### Encrypt with Custom Key

```bash
envcrypt encrypt --key "my-secret-key"
```

#### Encrypt Environment-Specific File

```bash
envcrypt encrypt --env production
# Encrypts .env.production to .env.production.encrypted
```

#### Encrypt Custom Path

```bash
envcrypt encrypt --input config/secrets.env --key "my-key"
# Encrypts config/secrets.env to config/secrets.env.encrypted
```

#### Decrypt with Key

```bash
envcrypt decrypt --key "my-secret-key"
```

#### Decrypt Custom Path

```bash
envcrypt decrypt --input .env.production.encrypted --key "my-key"
# Decrypts to .env.production
```

### Key Format

Keys can be provided with or without the `base64:` prefix:

```bash
# Both of these are equivalent:
envcrypt decrypt --key "base64:abc123..."
envcrypt decrypt --key "abc123..."
```

The tool will automatically strip the prefix if present.

## Security

### Encryption Details

- **Algorithm**: AES-256 in CBC mode
- **Authentication**: HMAC-SHA256
- **Key Derivation**: PBKDF2-HMAC-SHA256 with 100,000 iterations
- **IV Generation**: Cryptographically secure random 16-byte IV per encryption
- **Salt**: Random 16-byte salt per encryption (stored with encrypted data)

### Security Features

- **Constant-Time MAC Verification**: Prevents timing attacks during MAC verification
- **Authenticate-Then-Decrypt**: MAC is verified before decryption to prevent padding oracle attacks
- **Key Zeroization**: Sensitive keys are automatically cleared from memory after use
- **Random IVs**: Each encryption uses a unique random IV
- **Unique Salts**: Each encryption uses a unique random salt

### File Format

Encrypted files contain base64-encoded data with the following structure:

```
base64([Salt (16 bytes)][IV (16 bytes)][Encrypted Data][MAC (32 bytes)])
```

- **Salt**: Used for key derivation, stored with encrypted data
- **IV**: Initialization vector for AES-CBC, unique per encryption
- **Encrypted Data**: AES-256-CBC encrypted plaintext with PKCS7 padding
- **MAC**: HMAC-SHA256 of (IV + Encrypted Data)

### Best Practices

1. **Store Keys Securely**: Never commit encryption keys to version control
2. **Use Strong Keys**: Use randomly generated keys (the tool generates secure keys by default)
3. **Rotate Keys**: Periodically rotate encryption keys for sensitive data
4. **Protect Keys**: Store keys in a secure password manager or secret management system
5. **Limit Access**: Only share keys with authorized personnel
6. **Backup Keys**: Keep secure backups of encryption keys (you cannot decrypt without them)

## Testing

### Running Tests

#### Standard Cargo (Serial Execution)

```bash
cargo test
```

**Note:** Cargo runs integration test files serially (one after another), which can be slow.

#### cargo-nextest (Parallel Execution) - Recommended

For much faster test execution, use `cargo-nextest` which runs integration test files in parallel:

```bash
# Install cargo-nextest (one-time setup)
cargo install cargo-nextest --locked

# Run tests in parallel
cargo nextest run
```

**Performance:** With multiple test files, `cargo nextest run` is typically **10-20x faster** than `cargo test` because it runs all test files concurrently instead of sequentially.

### Test Structure

Tests are organized by feature:

- `tests/cli_tests/encrypt.rs` - Basic encrypt functionality
- `tests/cli_tests/decrypt.rs` - Basic decrypt functionality  
- `tests/cli_tests/roundtrip.rs` - Encrypt/decrypt roundtrip tests
- `tests/cli_tests/paths.rs` - Custom path and `--input` flag tests
- `tests/cli_tests/keys.rs` - Key parsing tests (base64 prefix, whitespace)
- `tests/cli_tests/env_flag.rs` - `--env` flag tests
- `tests/cli_tests/errors.rs` - Error condition tests
- `tests/common/mod.rs` - Shared test utilities

### Configuration

Test parallelization is configured in `.config/nextest.toml`. By default, tests run with as many threads as there are logical CPUs.

## Feature Flags

The project uses feature flags for modular compilation. Available features:

- `cipher`: Cryptographic operations (AES, HMAC, PBKDF2, etc.)
- `encrypt`: Enable encryption command
- `decrypt`: Enable decryption command
- `key-flag`: Enable `--key` flag for command-line key input
- `env-flag`: Enable `--env` flag for environment-specific files
- `input-flag`: Enable `--input` flag for custom input paths

Default features include all of the above. To build with specific features:

```bash
cargo build --no-default-features --features "cipher,encrypt,decrypt"
```

## API Documentation

Generate API documentation:

```bash
cargo doc --open
```

The library provides modules for:
- `cipher`: Cryptographic cipher implementations
- `key`: Key derivation and generation utilities
- `cli`: Command-line interface functions

## Contributing

Contributions are welcome! Please ensure that:

1. All tests pass (`cargo test`)
2. Code is properly documented
3. Security considerations are maintained
4. Changes are tested with both `cargo test` and `cargo nextest run`

## License

[Add your license information here]

## Acknowledgments

Built with Rust and using secure cryptographic libraries:
- `aes` and `cbc` for encryption
- `hmac` and `sha2` for authentication
- `pbkdf2` for key derivation
- `zeroize` for secure memory clearing
- `subtle` for constant-time operations
