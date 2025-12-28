# Changelog

All notable changes to RustOCR are documented in this file.

## [0.3.0] - 2025-12-28

### 🚀 Major Features

#### Server Mode
- **FastAPI persistent server** for 5-10x performance improvement
- HTTP-based client-server architecture
- Model caching eliminates reload overhead
- Server management commands: `--server`, `--server-stop`, `--server-status`
- Configurable port and host settings
- PID file tracking for reliable process management
- Reduces latency from 3-5s to <500ms for subsequent requests

#### Batch Processing
- Process multiple images with glob patterns (`--dir`)
- Visual progress indicators with elapsed time and ETA
- Graceful error handling for failed images
- Batch summary statistics (success/failure counts)
- Optional output directory for individual results (`--output-dir`)
- Compatible with both server and subprocess modes

### ✨ New Features

- **HTTP Client**: reqwest-based client for server communication
- **Health Checks**: Automatic server availability verification
- **Progress Bars**: Real-time batch processing status with indicatif
- **Error Recovery**: Batch processing continues despite individual failures

### 🔧 Improvements

- Made `--input` optional for server management commands
- Enhanced CLI with server and batch flags
- Improved documentation with SERVER_MODE.md guide
- Better error messages and user feedback

### 📦 Dependencies Added

- `reqwest` 0.11 - HTTP client
- `tokio` 1.0 - Async runtime
- `base64` 0.21 - Image encoding
- `glob` 0.3 - Pattern matching
- `indicatif` 0.17 - Progress bars

### 🐛 Bug Fixes

- Fixed argument requirements for server management
- Resolved input validation for different operation modes

### 📚 Documentation

- Comprehensive SERVER_MODE.md guide
- Usage examples for all new features
- Performance comparison tables
- Troubleshooting section

### 🔄 Breaking Changes

None - v0.3.0 is fully backward compatible with v0.2.0

---

## [0.2.0] - 2025-12-28

### Initial Release

- Subprocess-based Python bridge for EasyOCR
- Support for 80+ languages
- Multiple output formats (JSON, text, detailed)
- GPU/CPU mode selection
- CLI with clap argument parsing
- Comprehensive README documentation

---

## How to Upgrade

From v0.2.0 to v0.3.0:
```bash
cd rustOCR
git pull origin main
cargo build --release
```

No configuration changes required!
