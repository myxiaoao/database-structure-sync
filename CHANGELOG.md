# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-02-23

### Added
- Initial project setup with Tauri 2.x + React 19
- MySQL, PostgreSQL, and MariaDB database support
- Schema comparison engine with diff visualization
- SQL generation for schema synchronization
- Connection management with secure password storage (system keychain)
- SSH tunnel support for remote connections
- SSL/TLS encryption support
- Internationalization (English and Chinese)
- Dark/Light/System theme support
- Cross-platform builds (Windows, macOS, Linux)
- GitHub Actions CI/CD pipeline
- Comprehensive test suite

### Fixed
- Align frontend SSH/SSL types with backend nested structure

### Security
- Content Security Policy (CSP) configuration
- Capabilities-based permission model
- Secure credential storage using system keychain
