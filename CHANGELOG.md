# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.3.0] - 2026-03-27

### Added
- Timestamp in default SQL export filename to prevent overwriting previous exports

### Fixed
- Unified border-radius and overflow handling across components
- Diff tree readability, SQL preview panel, toolbar, and bottom bar polish
- Compare button contrast and toolbar font size fine-tuning
- Endpoint selector design: compact horizontal toolbar layout with polished spacing
- Flash on re-compare eliminated by keeping old results visible
- Compare button height alignment with endpoint selector cards

### Changed
- Redesigned main layout for clearer information hierarchy
- Redesigned endpoint selector as unified card with bottom action
- Unified horizontal padding to px-5 across header and content

## [1.2.0] - 2026-03-01

### Fixed
- Skip keychain reads in list_connections to prevent errors

### Changed
- Added macOS Gatekeeper workaround documentation
- Synced Cargo.lock version

## [1.1.0] - 2026-02-24

### Added
- Cross-database type warning when comparing incompatible databases (e.g. MySQL vs PostgreSQL)
- Connection form validation for required fields (name, host, port, username)
- Detailed column diff display showing full definition differences
- Debug logging for column comparison diagnostics

### Fixed
- Error toast showing "undefined" instead of actual error message from Tauri backend
- Duplicate key error caused by UNIQUE constraints being listed as both indexes and constraints in MySQL and PostgreSQL
- `MODIFY COLUMN` on nullable `timestamp` columns silently failing due to missing explicit `NULL DEFAULT NULL`
- False positive column diffs caused by `ordinal_position` comparison
- Missing blank line between SQL body and footer in exported SQL files
- Accidental connection creation triggered by pressing Enter in form input fields
- Empty connections being saved when required fields are blank

### Changed
- Default window size increased from 1200×800 to 1400×900
- Minimum window size increased from 900×600 to 1200×800

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
