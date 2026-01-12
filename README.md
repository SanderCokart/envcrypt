# envcrypt

> **⚠️ VIBE CODED ⚠️**  
> This project was fully vibe coded and AI generated. It was forced to work with test-driven development to ensure working order. While functional, it may not be up to production standards. Use at your own discretion.
> 
> **Note:** Just because it is vibe coded does not mean there was no human intervention. Each commit requires human intervention.

A secure command-line tool for encrypting and decrypting environment files using strong encryption algorithms with authentication.

**Inspired by Laravel's `php artisan env:encrypt` functionality.**

## Overview

`envcrypt` provides a simple and secure way to encrypt sensitive environment files (like `.env`) before committing them to version control. The encrypted files can be safely stored in repositories, and only users with the encryption key can decrypt them.

## Features

- **Multiple Cipher Support**: AES-256-CBC, AES-256-GCM, and ChaCha20-Poly1305
- **Strong Encryption**: Industry-standard encryption algorithms with authentication
- **Secure Key Derivation**: PBKDF2 with 100,000 iterations
- **Constant-Time Security**: Constant-time MAC verification to prevent timing attacks
- **Key Zeroization**: Automatic memory clearing of sensitive keys
- **Environment Support**: Support for multiple environments (local, production, etc.)
- **Flexible Paths**: Custom input/output paths
- **Verbosity Control**: Multiple output levels (`--silent`, `--quiet`, `--verbose`)
- **Non-Interactive Mode**: `--no-interaction` flag for automated workflows
- **File Management**: `--force` to overwrite files, `--prune` to delete originals after encryption
- **Feature Flags**: Modular compilation with optional features

## Installation

### Quick Install (Recommended)

#### Linux/macOS

Install with a single command:

```bash
curl -fsSL https://raw.githubusercontent.com/SanderCokart/envcrypt/main/install.sh | bash
```

#### Windows

Install with a single command:

```powershell
powershell -c "irm https://raw.githubusercontent.com/SanderCokart/envcrypt/main/install.ps1 | iex"
```

Or if you have execution policy restrictions:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://raw.githubusercontent.com/SanderCokart/envcrypt/main/install.ps1 | iex"
```

The install script will:
- Automatically detect your platform (OS and architecture)
- Download the latest pre-built binary from GitHub Releases (if available)
- Fall back to building from source if no binary is available (requires Rust)
- Install it to:
  - Linux/macOS: `~/.envcrypt/bin/envcrypt`
  - Windows: `%USERPROFILE%\.envcrypt\bin\envcrypt.exe`
- Add it to your PATH (in your shell config file on Linux/macOS, or user PATH on Windows)
- Automatically refresh your shell configuration

**Note:** After installation, you may need to reload your terminal (close and reopen, or run `source ~/.bashrc` / `source ~/.zshrc` etc.) for the `envcrypt` command to be available in new terminal sessions. The install script will add `envcrypt` to your PATH in the current session, but for it to persist in future sessions, you'll need to reload your terminal.

### Alternative: Install from Local Repository

If you've cloned the repository locally:

**Linux/macOS:**
```bash
./install.sh
```

**Windows:**
```powershell
.\install.ps1
```

The script will automatically detect the GitHub repository from the git remote. Otherwise, you can set the `ENVCRYPT_REPO` environment variable:

**Linux/macOS:**
```bash
export ENVCRYPT_REPO="SanderCokart/envcrypt"
./install.sh
```

**Windows:**
```powershell
$env:ENVCRYPT_REPO = "SanderCokart/envcrypt"
.\install.ps1
```

### Prerequisites

- **For pre-built binaries**: No prerequisites needed! The install script will download the binary automatically.
- **For building from source**: Rust and Cargo are required. If you don't have Rust installed, you can install it from [rustup.rs](https://rustup.rs/).

The install script will automatically fall back to building from source if:
- No pre-built binary is available for your platform
- The download fails
- The GitHub repository is not configured

### Using Cargo

```bash
cargo install --path .
```

Or if you have the repository locally:
```bash
cargo build --release
# Binary will be at target/release/envcrypt
```

### Uninstall

To uninstall `envcrypt`:

#### Linux/macOS

Uninstall with a single command:

```bash
curl -fsSL https://raw.githubusercontent.com/SanderCokart/envcrypt/main/install.sh | bash -s -- --uninstall
```

Or if you have the repository locally:

```bash
./install.sh --uninstall
```

#### Windows

Windows uninstall script coming soon.

#### Linux/macOS

The uninstall script will:
- Remove the `envcrypt` binary from `~/.envcrypt/bin/envcrypt`
- Remove the PATH configuration from your shell config file
- Remove the `.envcrypt` directory if it's empty

**Note:** You may need to restart your terminal for PATH changes to take effect after uninstallation.

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

#### Global Options

These options apply to both `encrypt` and `decrypt` commands:

- `--silent`: Do not output any message (suppresses all output including errors)
- `--force`: Overwrite existing encrypted/decrypted files without prompting
- `-q, --quiet`: Only errors are displayed. All other output is suppressed
- `-n, --no-interaction`: Do not ask any interactive question
  - For encryption: Automatically generates a new key if `--key` is not provided
  - For decryption: Requires `--key` to be provided (will error if missing)
- `-v, --verbose`: Increase the verbosity of messages
  - `-v`: Normal output (level 1)
  - `-vv`: More verbose output (level 2)
  - `-vvv`: Debug output (level 3)
- `-V, --version`: Display application version with release date

**Flag Precedence:**
- `--silent` overrides `--quiet` and `--verbose` (suppresses all output)
- `--quiet` suppresses info/verbose/debug messages but shows errors
- Verbosity levels increase detail: `-v` (normal) < `-vv` (verbose) < `-vvv` (debug)

#### Encryption Options

```bash
envcrypt encrypt [OPTIONS]
```

- `--cipher <CIPHER>`: Cipher to use (default: `AES-256-CBC`)
- `--key <KEY>`: Encryption key (if not provided, will prompt unless `--no-interaction` is used)
- `--input <PATH>`: Input file path (default: `.env`, or `.env.{env}` if `--env` is specified)
- `--env <ENV>`: Environment name (e.g., `local`, `production`). When specified:
  - Default input: `.env.{env}`
  - Default output: `.env.{env}.encrypted`
- `--prune`: Delete the original `.env` file after successful encryption (encrypt only)

#### Decryption Options

```bash
envcrypt decrypt [OPTIONS]
```

- `--cipher <CIPHER>`: Cipher to use (default: `AES-256-CBC`)
- `--key <KEY>`: Decryption key (if not provided, will prompt unless `--no-interaction` is used)
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

#### Encrypt with Different Cipher

```bash
# Use AES-256-GCM
envcrypt encrypt --cipher AES-256-GCM

# Use ChaCha20-Poly1305
envcrypt encrypt --cipher CHACHA20-POLY1305
```

#### Decrypt with Specific Cipher

```bash
# When decrypting, use the same cipher that was used for encryption
envcrypt decrypt --cipher AES-256-GCM --key "my-key"
```

#### Non-Interactive Encryption

```bash
envcrypt encrypt --no-interaction
# Automatically generates a new key without prompting
```

#### Non-Interactive Decryption

```bash
envcrypt decrypt --no-interaction --key "my-secret-key"
# Decrypts without prompting (key must be provided)
```

#### Force Overwrite Existing Files

```bash
envcrypt encrypt --force
# Overwrites .env.encrypted if it already exists

envcrypt decrypt --force
# Overwrites .env if it already exists
```

#### Prune Original File After Encryption

```bash
envcrypt encrypt --prune
# Encrypts .env to .env.encrypted and deletes the original .env file
```

#### Quiet Mode (Errors Only)

```bash
envcrypt encrypt --quiet
# Only shows errors, suppresses all other output
```

#### Silent Mode (No Output)

```bash
envcrypt encrypt --silent
# Suppresses all output including errors
```

#### Verbose Output

```bash
envcrypt encrypt -v      # Normal verbose output
envcrypt encrypt -vv      # More verbose output
envcrypt encrypt -vvv     # Debug output
```

#### Display Version

```bash
envcrypt --version
# Output: envcrypt 0.1.0 (2026-01-11)
```

### Key Format

Keys can be provided with or without the `base64:` prefix:

```bash
# Both of these are equivalent:
envcrypt decrypt --key "base64:abc123..."
envcrypt decrypt --key "abc123..."
```

The tool will automatically strip the prefix if present.

### Available Ciphers

`envcrypt` supports multiple encryption algorithms. Choose the cipher that best fits your needs:

#### AES-256-CBC (Default)
- **Algorithm**: AES-256 in CBC mode with PKCS7 padding
- **Authentication**: HMAC-SHA256 (separate MAC)
- **Format**: `[Salt][IV (16 bytes)][Encrypted Data][MAC (32 bytes)]`
- **Use Case**: Default choice, widely compatible, proven security
- **Example**: `envcrypt encrypt --cipher AES-256-CBC`

#### AES-256-GCM
- **Algorithm**: AES-256 in Galois/Counter Mode (GCM)
- **Authentication**: Built-in GCM authentication tag
- **Format**: `[Salt][Nonce (12 bytes)][Encrypted Data][Tag (16 bytes)]`
- **Use Case**: Modern authenticated encryption, hardware-accelerated on modern CPUs
- **Example**: `envcrypt encrypt --cipher AES-256-GCM`

#### ChaCha20-Poly1305
- **Algorithm**: ChaCha20 stream cipher with Poly1305 MAC
- **Authentication**: Built-in Poly1305 authentication tag
- **Format**: `[Salt][Nonce (12 bytes)][Encrypted Data][Tag (16 bytes)]`
- **Use Case**: Fast software implementation, excellent performance without hardware acceleration
- **Example**: `envcrypt encrypt --cipher CHACHA20-POLY1305`

**Note**: When decrypting, you must use the same cipher that was used for encryption. The cipher name is case-insensitive.

## Security

### Encryption Details

- **Algorithms**: AES-256-CBC, AES-256-GCM, or ChaCha20-Poly1305
- **Authentication**: 
  - AES-256-CBC: HMAC-SHA256 (separate MAC)
  - AES-256-GCM: Built-in GCM authentication tag
  - ChaCha20-Poly1305: Built-in Poly1305 authentication tag
- **Key Derivation**: PBKDF2-HMAC-SHA256 with 100,000 iterations
- **IV/Nonce Generation**: Cryptographically secure random values per encryption
  - AES-256-CBC: 16-byte IV
  - AES-256-GCM: 12-byte nonce
  - ChaCha20-Poly1305: 12-byte nonce
- **Salt**: Random 16-byte salt per encryption (stored with encrypted data)

### Security Features

- **Constant-Time MAC Verification**: Prevents timing attacks during MAC verification
- **Authenticate-Then-Decrypt**: MAC is verified before decryption to prevent padding oracle attacks
- **Key Zeroization**: Sensitive keys are automatically cleared from memory after use
- **Random IVs**: Each encryption uses a unique random IV
- **Unique Salts**: Each encryption uses a unique random salt

### File Format

Encrypted files contain base64-encoded data. The structure varies by cipher:

**AES-256-CBC:**
```
base64([Salt (16 bytes)][IV (16 bytes)][Encrypted Data][MAC (32 bytes)])
```

**AES-256-GCM:**
```
base64([Salt (16 bytes)][Nonce (12 bytes)][Encrypted Data][Tag (16 bytes)])
```

**ChaCha20-Poly1305:**
```
base64([Salt (16 bytes)][Nonce (12 bytes)][Encrypted Data][Tag (16 bytes)])
```

- **Salt**: Used for key derivation, stored with encrypted data (all ciphers)
- **IV/Nonce**: Initialization vector or nonce, unique per encryption
- **Encrypted Data**: Encrypted plaintext
- **MAC/Tag**: Authentication tag (format depends on cipher)

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
- `tests/cli_tests/flags.rs` - Global flags tests (`--silent`, `--force`, `--quiet`, `--prune`, `--no-interaction`, `--verbose`, `--version`)
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
