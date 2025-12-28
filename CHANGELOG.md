# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2025-12-28

### Added
- PDF support module for text extraction and processing
- Parallel processing with rayon for concurrent batch operations
- CI/CD pipeline with GitHub Actions
- Docker support with multi-stage builds
- Docker Compose configuration
- Integration tests
- Crates.io publication support
- Enhanced REST API server with comprehensive endpoints
- API documentation (Swagger UI and ReDoc)
- Server statistics and monitoring
- Request tracking middleware

### Changed
- Updated dependencies: added image, pdf-extract, rayon, num_cpus
- Enhanced Cargo.toml metadata for crates.io
- Improved error handling across modules

### Documentation
- Added PUBLISHING.md guide
- Added API_DOCUMENTATION.md
- Created comprehensive CI/CD workflows
- Docker deployment guides

## [0.4.0] - 2025-12-28

### Added
- TOML-based configuration file support
- Named profiles for common workflows
- Configuration hierarchy (CLI > project > user > system)
- `--init-config` command to generate default config
- `--profile` flag to use named profiles
- `--config` flag for custom config paths

### Configuration Locations
- System: `/etc/rustocr/config.toml`
- User: `~/.config/rustocr/config.toml`
- Project: `./rustocr.toml`

## [0.3.0] - 2025-12-28

### Added
- **Server Mode** - FastAPI persistent server for 5-10x performance
  - HTTP-based client-server architecture
  - Model caching eliminates reload overhead
  - `--server`, `--server-stop`, `--server-status` commands
  
- **Batch Processing**
  - Process multiple images with glob patterns (`--dir`)
  - Visual progress indicators with indicatif
  - Error handling for individual failures
  - Optional output directory (`--output-dir`)

- **HTTP Client**
  - reqwest-based implementation
  - Health checks
  - Base64 image encoding

### Performance
- Achieved <500ms latency for server mode
- 5-10x faster than subprocess mode

## [0.2.0] - Initial Release

### Added
- Basic CLI wrapper for EasyOCR
- Subprocess-based Python bridge
- Support for 80+ languages
- Multiple output formats (JSON, text, detailed)
- GPU/CPU mode selection
- Confidence threshold filtering
- Basic error handling

### Features
- Language selection
- Image input
- Output formatting
- Detailed/simple modes

[0.5.0]: https://github.com/cyberiums/RustOCR/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/cyberiums/RustOCR/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/cyberiums/RustOCR/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/cyberiums/RustOCR/releases/tag/v0.2.0
