# Database Structure Sync

A cross-platform desktop application for comparing and synchronizing database table structures between MySQL, PostgreSQL, and MariaDB databases.

[![Version](https://img.shields.io/github/v/release/myxiaoao/database-structure-sync?label=Version)](https://github.com/myxiaoao/database-structure-sync/releases)
[![CI](https://github.com/myxiaoao/database-structure-sync/actions/workflows/build.yml/badge.svg)](https://github.com/myxiaoao/database-structure-sync/actions/workflows/build.yml)
[![Release](https://github.com/myxiaoao/database-structure-sync/actions/workflows/release.yml/badge.svg)](https://github.com/myxiaoao/database-structure-sync/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.x-24C8D8?logo=tauri)](https://tauri.app)
[![React](https://img.shields.io/badge/React-19-61DAFB?logo=react)](https://react.dev)

## Features

- **Multi-Database Support**: MySQL, PostgreSQL, and MariaDB
- **Visual Schema Comparison**: Side-by-side diff view of database structures
- **Comprehensive Diff Detection**:
  - Tables: add / drop
  - Columns: add / drop / modify (type, nullable, default, etc.)
  - Indexes: add / drop / modify
  - Foreign Keys: add / drop / modify
  - Unique Constraints: add / drop / modify
- **Selective Sync**: Choose which changes to apply
- **SQL Preview**: Review generated SQL before execution
- **SQL Export**: Export sync scripts to `.sql` files
- **Secure Connections**:
  - SSH tunnel support for secure remote connections
  - SSL/TLS encryption support
  - Passwords stored in system keychain
- **Auto Update**: In-app update notifications via Tauri updater plugin (requires signing key configuration)
- **Cross-Platform**: Windows, macOS, and Linux
- **Internationalization**: English and Chinese language support
- **Dark/Light Mode**: Automatic theme detection with manual override

## Installation

### Pre-built Binaries

Download the latest release for your platform from the [Releases](https://github.com/myxiaoao/database-structure-sync/releases) page:

- **macOS**: `.dmg` (Intel and Apple Silicon)
- **Windows**: `.msi` or `.exe` installer
- **Linux**: `.deb`, `.rpm`, or `.AppImage`

### Build from Source

#### Prerequisites

- [Node.js](https://nodejs.org/) 20.x or later
- [Rust](https://www.rust-lang.org/) 1.85 or later
- Platform-specific dependencies:

**macOS:**
```bash
xcode-select --install
```

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

**Windows:**
- Visual Studio Build Tools with C++ workload
- WebView2 Runtime (usually pre-installed on Windows 10/11)

#### Build Steps

```bash
# Clone the repository
git clone https://github.com/myxiaoao/database-structure-sync.git
cd database-structure-sync

# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

The built application will be in `src-tauri/target/release/bundle/`.

## Usage

### Adding a Connection

1. Click "New Connection" in the sidebar
2. Fill in the connection details:
   - **Name**: A friendly name for the connection
   - **Database Type**: MySQL, PostgreSQL, or MariaDB
   - **Host/Port**: Database server address
   - **Username/Password**: Database credentials
   - **Database**: Target database name
3. (Optional) Configure SSH tunnel or SSL settings
4. Click "Test Connection" to verify
5. Click "Save" to store the connection

### Comparing Databases

1. Select a **Source** database (the reference schema)
2. Select a **Target** database (the one to be updated)
3. Click "Start Compare"
4. Review the differences in the diff tree
5. Select the changes you want to apply
6. Preview the SQL statements
7. Click "Execute Sync" to apply changes

## Architecture

```
├── src/                    # React frontend
│   ├── components/         # UI components
│   ├── hooks/              # React hooks
│   ├── lib/                # Utilities and API
│   ├── locales/            # i18n translations
│   ├── types/              # TypeScript type definitions
│   └── test/               # Test utilities and setup
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── db/             # Database drivers & SQL generators
│   │   ├── diff/           # Schema comparison
│   │   ├── models/         # Data models
│   │   ├── ssh/            # SSH tunnel
│   │   ├── storage/        # Config storage
│   │   └── error.rs        # Error types
│   └── tests/              # Integration tests
└── public/                 # Static assets
```

## Tech Stack

### Frontend
- React 19 with TypeScript
- Tailwind CSS 4
- shadcn/ui components
- react-i18next for internationalization

### Backend
- Rust (edition 2024) with Tauri 2.x
- SQLx for database connections
- Tokio async runtime
- russh for SSH tunnels
- keyring for system keychain integration
- thiserror / anyhow for error handling
- tauri-plugin-updater for auto updates (disabled until signing key is configured)
- tauri-plugin-dialog for native dialogs

## Development

### Running Tests

```bash
# Frontend tests
npm run test

# Rust tests
cd src-tauri && cargo test

# Type checking
npm run build
```

### Linting & Formatting

```bash
# Lint frontend code
npm run lint
npm run lint:fix

# Format frontend code
npm run format
npm run format:check

# Lint Rust code
cd src-tauri && cargo clippy

# Format Rust code
cd src-tauri && cargo fmt
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Tauri](https://tauri.app/) - Cross-platform app framework
- [shadcn/ui](https://ui.shadcn.com/) - UI components
- [SQLx](https://github.com/launchbadge/sqlx) - Rust SQL toolkit
