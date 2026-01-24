# Database Structure Synchronization Tool - Design Document

## Overview

A desktop application for comparing and synchronizing database table structures between source and target databases. Built with Tauri framework for cross-platform support.

## Target Users

Individual developers who need to sync database structures between development, testing, and production environments.

## Supported Databases

Phase 1:
- MySQL
- PostgreSQL
- MariaDB

## Features

### Core Features

1. **Structure Comparison** - Compare table structures between two databases
2. **One-way Sync** - Sync from source to target database
3. **SQL Preview & Execute** - Preview generated SQL and execute directly
4. **Connection Management** - Save connections locally with encrypted passwords

### Sync Objects

Table structures only:
- Columns (name, type, nullable, default, auto_increment, comment)
- Primary keys
- Indexes
- Foreign keys
- Unique constraints

### Connection Features

- SSH Tunnel (password / private key authentication)
- SSL/TLS certificate authentication
- Local storage with encrypted passwords

### UI Features

- Tree view for diff results (default all selected)
- SQL preview panel
- Language switch (English default, Simplified Chinese)

## Technical Stack

### Frontend
- React 18
- TypeScript
- Tailwind CSS
- shadcn/ui
- react-i18next

### Backend
- Rust
- Tauri 2.x
- sqlx (async database driver)
- russh (SSH client)
- keyring (system keychain)

### Local Storage
- SQLite (connection configs)
- System keychain (passwords)

## Data Models

```rust
// Database connection configuration
struct Connection {
    id: String,
    name: String,
    db_type: DbType,            // MySQL | PostgreSQL | MariaDB
    host: String,
    port: u16,
    username: String,
    password: String,           // encrypted
    database: String,
    ssh_config: Option<SshConfig>,
    ssl_config: Option<SslConfig>,
    created_at: DateTime,
    updated_at: DateTime,
}

enum DbType {
    MySQL,
    PostgreSQL,
    MariaDB,
}

struct SshConfig {
    enabled: bool,
    host: String,
    port: u16,                  // default 22
    username: String,
    auth_method: SshAuthMethod,
}

enum SshAuthMethod {
    Password { password: String },
    PrivateKey {
        private_key_path: String,
        passphrase: Option<String>,
    },
}

struct SslConfig {
    enabled: bool,
    ca_cert_path: Option<String>,
    client_cert_path: Option<String>,
    client_key_path: Option<String>,
    verify_server: bool,
}

// Table schema definition
struct TableSchema {
    name: String,
    columns: Vec<Column>,
    primary_key: Option<PrimaryKey>,
    indexes: Vec<Index>,
    foreign_keys: Vec<ForeignKey>,
    unique_constraints: Vec<UniqueConstraint>,
}

struct Column {
    name: String,
    data_type: String,
    nullable: bool,
    default_value: Option<String>,
    auto_increment: bool,
    comment: Option<String>,
    ordinal_position: u32,
}

struct PrimaryKey {
    name: Option<String>,
    columns: Vec<String>,
}

struct Index {
    name: String,
    columns: Vec<String>,
    unique: bool,
    index_type: String,         // BTREE, HASH, etc.
}

struct ForeignKey {
    name: String,
    columns: Vec<String>,
    ref_table: String,
    ref_columns: Vec<String>,
    on_delete: String,
    on_update: String,
}

struct UniqueConstraint {
    name: String,
    columns: Vec<String>,
}

// Diff types
enum DiffType {
    TableAdded,
    TableRemoved,
    ColumnAdded,
    ColumnRemoved,
    ColumnModified,
    IndexAdded,
    IndexRemoved,
    IndexModified,
    ForeignKeyAdded,
    ForeignKeyRemoved,
    UniqueConstraintAdded,
    UniqueConstraintRemoved,
}

struct DiffItem {
    diff_type: DiffType,
    table_name: String,
    object_name: Option<String>,
    source_def: Option<String>,
    target_def: Option<String>,
    sql: String,
    selected: bool,             // default true
}
```

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                   Tauri Application                  │
├────────────────────────┬────────────────────────────┤
│    Frontend (Webview)  │      Backend (Rust)        │
│  ┌──────────────────┐  │  ┌──────────────────────┐  │
│  │ React + shadcn   │  │  │  Tauri Commands      │  │
│  │ - ConnectionPage │◄─┼─►│  - connection::*     │  │
│  │ - SyncPage       │  │  │  - schema::*         │  │
│  │ - DiffTreeView   │  │  │  - diff::*           │  │
│  │ - SqlPreview     │  │  │  - sync::*           │  │
│  └──────────────────┘  │  └──────────────────────┘  │
│                        │  ┌──────────────────────┐  │
│                        │  │  Database Drivers    │  │
│                        │  │  - sqlx (async)      │  │
│                        │  │  - MySQL/PG/MariaDB  │  │
│                        │  └──────────────────────┘  │
│                        │  ┌──────────────────────┐  │
│                        │  │  SSH Client          │  │
│                        │  │  - russh             │  │
│                        │  │  - Port forwarding   │  │
│                        │  └──────────────────────┘  │
│                        │  ┌──────────────────────┐  │
│                        │  │  Local Storage       │  │
│                        │  │  - SQLite (config)   │  │
│                        │  │  - Keychain (pwd)    │  │
│                        │  └──────────────────────┘  │
└────────────────────────┴────────────────────────────┘
```

## Project Structure

```
database-structure-synchronization/
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── connection.rs     # Connection management
│   │   │   ├── schema.rs         # Schema reading
│   │   │   ├── diff.rs           # Diff calculation
│   │   │   └── sync.rs           # Execute sync
│   │   ├── db/
│   │   │   ├── mod.rs
│   │   │   ├── mysql.rs          # MySQL/MariaDB driver
│   │   │   ├── postgres.rs       # PostgreSQL driver
│   │   │   └── schema.rs         # Schema parsing
│   │   ├── ssh/
│   │   │   ├── mod.rs
│   │   │   └── tunnel.rs         # SSH tunnel
│   │   ├── diff/
│   │   │   ├── mod.rs
│   │   │   ├── comparator.rs     # Structure comparison
│   │   │   └── sql_generator.rs  # SQL generation
│   │   ├── storage/
│   │   │   ├── mod.rs
│   │   │   ├── config.rs         # Connection config storage
│   │   │   └── crypto.rs         # Password encryption
│   │   └── models/
│   │       └── mod.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                          # React frontend
│   ├── components/
│   │   ├── ui/                   # shadcn components
│   │   ├── ConnectionManager/
│   │   ├── ConnectionForm/
│   │   ├── DiffTree/
│   │   └── SqlPreview/
│   ├── hooks/
│   ├── lib/
│   ├── locales/
│   │   ├── en-US.json
│   │   └── zh-CN.json
│   ├── pages/
│   ├── App.tsx
│   └── main.tsx
├── docs/
│   └── plans/
├── package.json
└── tailwind.config.js
```

## UI Layout

```
┌─────────────────────────────────────────────────────────────┐
│  Database Structure Sync                    [EN ▼]  ─ □ ✕  │
├──────────────┬──────────────────────────────────────────────┤
│              │                                              │
│  Connections │   ┌─────────────────┐  ┌─────────────────┐   │
│  ──────────  │   │  Source         │  │  Target         │   │
│  ▼ Dev       │   │  [Select ▼]     │  │  [Select ▼]     │   │
│    └ dev_db  │   │  dev_db         │  │  test_db        │   │
│  ▼ Test      │   └─────────────────┘  └─────────────────┘   │
│    └ test_db │                                              │
│              │           [ Start Compare ]                  │
│  ──────────  │──────────────────────────────────────────────│
│  + New       │                                              │
│              │  Differences (12)          [All] [None]      │
│              │  ┌───────────────────────────────────────┐   │
│              │  │ ☑ 📁 users (3 changes)               │   │
│              │  │   ├ ☑ ➕ Add column: avatar_url      │   │
│              │  │   ├ ☑ ✏️ Modify column: email        │   │
│              │  │   └ ☑ ➕ Add index: idx_created_at   │   │
│              │  │ ☑ 📁 orders (2 changes)              │   │
│              │  │   ├ ☑ ➕ Add column: discount        │   │
│              │  │   └ ☐ ➖ Drop column: legacy_field   │   │
│              │  │ ☑ 📁 products (new table)            │   │
│              │  └───────────────────────────────────────┘   │
│              │                                              │
│              │  ┌─────────────────────────────────────────┐ │
│              │  │ -- SQL Preview                          │ │
│              │  │ ALTER TABLE users                       │ │
│              │  │   ADD COLUMN avatar_url VARCHAR(255);   │ │
│              │  └─────────────────────────────────────────┘ │
│              │                                              │
│              │                    [ Execute Sync ]          │
└──────────────┴──────────────────────────────────────────────┘
```

## Workflow

### 1. Connection Management

```
User clicks "New" → Connection form opens
    ↓
Fill basic info (name, type, host, port, user, password, database)
    ↓
Optional: Configure SSH tunnel
    ↓
Optional: Configure SSL/TLS
    ↓
Click "Test Connection" → Backend tests connection
    ↓
Click "Save" → Store config (password encrypted)
```

### 2. Structure Comparison

```
Select source and target connections → Click "Start Compare"
    ↓
┌─────────────────────────────────────────────────────┐
│  Backend parallel fetch schemas (information_schema) │
│  MySQL/MariaDB: SHOW CREATE TABLE + system tables    │
│  PostgreSQL: pg_catalog system tables                │
└─────────────────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────────────────┐
│  Diff algorithm:                                     │
│  1. Compare table lists → find added/removed tables  │
│  2. Compare columns per table → find column changes  │
│  3. Compare indexes/constraints → find index changes │
│  4. Generate SQL for each diff item                  │
└─────────────────────────────────────────────────────┘
    ↓
Frontend renders tree view, all items selected by default
```

### 3. SQL Generation & Execution

```
User toggles diff items → Preview SQL updates in real-time
    ↓
Click "Execute Sync" → Confirmation dialog
    ↓
Backend executes SQL statements in order
    ↓
Show execution result (success / error message)
```

## Key Implementation Notes

### 1. Password Encryption
- Primary: System keychain (macOS Keychain / Windows Credential Manager)
- Fallback: AES-256-GCM encryption with machine-derived key

### 2. Cross-Database Compatibility
```rust
trait SchemaReader {
    async fn get_tables(&self) -> Result<Vec<TableSchema>>;
    async fn test_connection(&self) -> Result<()>;
}

trait SqlGenerator {
    fn generate_add_column(&self, table: &str, col: &Column) -> String;
    fn generate_drop_column(&self, table: &str, col_name: &str) -> String;
    fn generate_modify_column(&self, table: &str, col: &Column) -> String;
    fn generate_add_index(&self, table: &str, idx: &Index) -> String;
    fn generate_drop_index(&self, table: &str, idx_name: &str) -> String;
    fn generate_create_table(&self, table: &TableSchema) -> String;
    fn generate_drop_table(&self, table_name: &str) -> String;
    // ...
}
```

### 3. SSH Tunnel Implementation
```rust
// Connection flow with SSH
async fn connect_with_ssh(config: &Connection) -> Result<DbConnection> {
    if let Some(ssh) = &config.ssh_config {
        // 1. Establish SSH connection
        let ssh_session = create_ssh_session(ssh).await?;

        // 2. Create local port forwarding
        let local_port = ssh_session.forward_port(
            config.host.clone(),
            config.port
        ).await?;

        // 3. Connect to database via local port
        connect_db("127.0.0.1", local_port, config).await
    } else {
        connect_db(&config.host, config.port, config).await
    }
}
```

### 4. SQL Execution Order
To handle dependencies correctly:
1. Drop foreign keys first
2. Drop indexes
3. Modify table structures (add/drop/modify columns)
4. Recreate indexes
5. Recreate foreign keys
6. Create new tables (with their indexes and constraints)

### 5. Error Handling
- All database operations use `Result` type
- Execute SQL statements one by one
- Stop on first error and report
- Frontend shows user-friendly error messages

## Dependencies

### Rust (Cargo.toml)
```toml
[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.7", features = ["runtime-tokio", "mysql", "postgres", "sqlite"] }
tokio = { version = "1", features = ["full"] }
russh = "0.43"
russh-keys = "0.43"
keyring = "2"
thiserror = "1"
anyhow = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
```

### Frontend (package.json)
```json
{
  "dependencies": {
    "react": "^18.0.0",
    "react-dom": "^18.0.0",
    "@tauri-apps/api": "^2.0.0",
    "i18next": "^23.0.0",
    "react-i18next": "^14.0.0",
    "tailwindcss": "^3.0.0",
    "class-variance-authority": "^0.7.0",
    "clsx": "^2.0.0",
    "lucide-react": "^0.300.0"
  },
  "devDependencies": {
    "@types/react": "^18.0.0",
    "@types/react-dom": "^18.0.0",
    "typescript": "^5.0.0",
    "vite": "^5.0.0",
    "@vitejs/plugin-react": "^4.0.0"
  }
}
```

## Future Considerations (Out of Scope for Phase 1)

- Oracle / SQL Server support
- Views, stored procedures, triggers sync
- Sync history and rollback
- Team collaboration features
- Schema versioning
