# Database Structure Sync - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Tauri-based desktop application for comparing and synchronizing database table structures between MySQL, PostgreSQL, and MariaDB databases.

**Architecture:** Tauri 2.x with React frontend (shadcn/ui) and Rust backend. Frontend handles UI/i18n, backend handles database connections, schema reading, diff calculation, and SQL execution. SQLite stores connection configs locally.

**Tech Stack:** Rust, Tauri 2.x, sqlx, russh, React 18, TypeScript, Tailwind CSS, shadcn/ui, react-i18next

---

## Phase 1: Project Setup

### Task 1.1: Initialize Tauri Project

**Files:**
- Create: `package.json`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`

**Step 1: Create Tauri project with React template**

Run:
```bash
npm create tauri-app@latest . -- --template react-ts --manager npm
```

Expected: Project scaffolded with Tauri + React + TypeScript

**Step 2: Verify project structure**

Run:
```bash
ls -la && ls -la src-tauri/src/
```

Expected: See package.json, src-tauri/, src/ directories

**Step 3: Commit**

```bash
git add -A
git commit -m "chore: initialize tauri project with react-ts template"
```

---

### Task 1.2: Configure Rust Dependencies

**Files:**
- Modify: `src-tauri/Cargo.toml`

**Step 1: Update Cargo.toml with required dependencies**

Replace the `[dependencies]` section in `src-tauri/Cargo.toml`:

```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-opener = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.8", features = ["runtime-tokio", "tls-rustls", "mysql", "postgres", "sqlite"] }
tokio = { version = "1", features = ["full"] }
russh = "0.46"
russh-keys = "0.46"
keyring = { version = "3", features = ["apple-native", "windows-native", "sync-secret-service"] }
thiserror = "2"
anyhow = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
async-trait = "0.1"
```

**Step 2: Build to verify dependencies resolve**

Run:
```bash
cd src-tauri && cargo check
```

Expected: Dependencies resolve successfully (may take time on first run)

**Step 3: Commit**

```bash
git add src-tauri/Cargo.toml
git commit -m "chore: add rust dependencies for database, ssh, and encryption"
```

---

### Task 1.3: Configure Frontend Dependencies

**Files:**
- Modify: `package.json`

**Step 1: Install shadcn/ui prerequisites**

Run:
```bash
npm install tailwindcss @tailwindcss/vite clsx tailwind-merge class-variance-authority lucide-react
```

**Step 2: Install i18n dependencies**

Run:
```bash
npm install i18next react-i18next
```

**Step 3: Install shadcn/ui**

Run:
```bash
npx shadcn@latest init -d
```

Select: New York style, Zinc color, CSS variables: yes

**Step 4: Add required shadcn components**

Run:
```bash
npx shadcn@latest add button input label select card dialog checkbox scroll-area dropdown-menu separator tabs textarea tree
```

**Step 5: Commit**

```bash
git add -A
git commit -m "chore: add frontend dependencies - shadcn/ui, tailwind, i18next"
```

---

### Task 1.4: Setup i18n Structure

**Files:**
- Create: `src/lib/i18n.ts`
- Create: `src/locales/en-US.json`
- Create: `src/locales/zh-CN.json`
- Modify: `src/main.tsx`

**Step 1: Create i18n configuration**

Create `src/lib/i18n.ts`:

```typescript
import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import enUS from '../locales/en-US.json';
import zhCN from '../locales/zh-CN.json';

const savedLang = localStorage.getItem('language') || 'en';

i18n.use(initReactI18next).init({
  resources: {
    en: { translation: enUS },
    zh: { translation: zhCN },
  },
  lng: savedLang,
  fallbackLng: 'en',
  interpolation: {
    escapeValue: false,
  },
});

export default i18n;
```

**Step 2: Create English locale**

Create `src/locales/en-US.json`:

```json
{
  "app": {
    "title": "Database Structure Sync"
  },
  "connection": {
    "title": "Connections",
    "new": "New Connection",
    "edit": "Edit",
    "delete": "Delete",
    "test": "Test Connection",
    "save": "Save",
    "cancel": "Cancel",
    "name": "Connection Name",
    "type": "Database Type",
    "host": "Host",
    "port": "Port",
    "username": "Username",
    "password": "Password",
    "database": "Database",
    "testSuccess": "Connection successful",
    "testFailed": "Connection failed"
  },
  "ssh": {
    "title": "SSH Tunnel",
    "enabled": "Enable SSH Tunnel",
    "host": "SSH Host",
    "port": "SSH Port",
    "username": "SSH Username",
    "authMethod": "Authentication Method",
    "password": "Password",
    "privateKey": "Private Key",
    "privateKeyPath": "Private Key Path",
    "passphrase": "Passphrase"
  },
  "ssl": {
    "title": "SSL/TLS",
    "enabled": "Enable SSL/TLS",
    "caCert": "CA Certificate",
    "clientCert": "Client Certificate",
    "clientKey": "Client Key",
    "verifyServer": "Verify Server Certificate"
  },
  "sync": {
    "source": "Source Database",
    "target": "Target Database",
    "selectConnection": "Select connection...",
    "compare": "Start Compare",
    "execute": "Execute Sync",
    "selectAll": "Select All",
    "deselectAll": "Deselect All",
    "noChanges": "No structural differences found",
    "changes": "differences"
  },
  "diff": {
    "tableAdded": "New Table",
    "tableRemoved": "Drop Table",
    "columnAdded": "Add Column",
    "columnRemoved": "Drop Column",
    "columnModified": "Modify Column",
    "indexAdded": "Add Index",
    "indexRemoved": "Drop Index",
    "indexModified": "Modify Index",
    "foreignKeyAdded": "Add Foreign Key",
    "foreignKeyRemoved": "Drop Foreign Key",
    "uniqueAdded": "Add Unique Constraint",
    "uniqueRemoved": "Drop Unique Constraint"
  },
  "sql": {
    "preview": "SQL Preview",
    "empty": "Select items to preview SQL"
  },
  "common": {
    "confirm": "Confirm",
    "cancel": "Cancel",
    "loading": "Loading...",
    "error": "Error",
    "success": "Success"
  }
}
```

**Step 3: Create Chinese locale**

Create `src/locales/zh-CN.json`:

```json
{
  "app": {
    "title": "数据库结构同步"
  },
  "connection": {
    "title": "连接管理",
    "new": "新建连接",
    "edit": "编辑",
    "delete": "删除",
    "test": "测试连接",
    "save": "保存",
    "cancel": "取消",
    "name": "连接名称",
    "type": "数据库类型",
    "host": "主机地址",
    "port": "端口",
    "username": "用户名",
    "password": "密码",
    "database": "数据库名",
    "testSuccess": "连接成功",
    "testFailed": "连接失败"
  },
  "ssh": {
    "title": "SSH 隧道",
    "enabled": "启用 SSH 隧道",
    "host": "SSH 主机",
    "port": "SSH 端口",
    "username": "SSH 用户名",
    "authMethod": "认证方式",
    "password": "密码",
    "privateKey": "私钥",
    "privateKeyPath": "私钥路径",
    "passphrase": "私钥密码"
  },
  "ssl": {
    "title": "SSL/TLS",
    "enabled": "启用 SSL/TLS",
    "caCert": "CA 证书",
    "clientCert": "客户端证书",
    "clientKey": "客户端密钥",
    "verifyServer": "验证服务器证书"
  },
  "sync": {
    "source": "源数据库",
    "target": "目标数据库",
    "selectConnection": "选择连接...",
    "compare": "开始比较",
    "execute": "执行同步",
    "selectAll": "全选",
    "deselectAll": "全不选",
    "noChanges": "未发现结构差异",
    "changes": "项差异"
  },
  "diff": {
    "tableAdded": "新建表",
    "tableRemoved": "删除表",
    "columnAdded": "新增列",
    "columnRemoved": "删除列",
    "columnModified": "修改列",
    "indexAdded": "新增索引",
    "indexRemoved": "删除索引",
    "indexModified": "修改索引",
    "foreignKeyAdded": "新增外键",
    "foreignKeyRemoved": "删除外键",
    "uniqueAdded": "新增唯一约束",
    "uniqueRemoved": "删除唯一约束"
  },
  "sql": {
    "preview": "SQL 预览",
    "empty": "选择项目以预览 SQL"
  },
  "common": {
    "confirm": "确认",
    "cancel": "取消",
    "loading": "加载中...",
    "error": "错误",
    "success": "成功"
  }
}
```

**Step 4: Import i18n in main.tsx**

Add to the top of `src/main.tsx` (after React imports):

```typescript
import './lib/i18n';
```

**Step 5: Commit**

```bash
git add src/lib/i18n.ts src/locales/ src/main.tsx
git commit -m "feat: add i18n setup with english and chinese locales"
```

---

## Phase 2: Rust Backend - Data Models

### Task 2.1: Create Data Models

**Files:**
- Create: `src-tauri/src/models/mod.rs`
- Create: `src-tauri/src/models/connection.rs`
- Create: `src-tauri/src/models/schema.rs`
- Create: `src-tauri/src/models/diff.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Create models directory**

Run:
```bash
mkdir -p src-tauri/src/models
```

**Step 2: Create connection models**

Create `src-tauri/src/models/connection.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DbType {
    MySQL,
    PostgreSQL,
    MariaDB,
}

impl DbType {
    pub fn default_port(&self) -> u16 {
        match self {
            DbType::MySQL | DbType::MariaDB => 3306,
            DbType::PostgreSQL => 5432,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SshAuthMethod {
    Password { password: String },
    PrivateKey {
        private_key_path: String,
        passphrase: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: SshAuthMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslConfig {
    pub enabled: bool,
    pub ca_cert_path: Option<String>,
    pub client_cert_path: Option<String>,
    pub client_key_path: Option<String>,
    pub verify_server: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: String,
    pub name: String,
    pub db_type: DbType,
    pub host: String,
    pub port: u16,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub database: String,
    pub ssh_config: Option<SshConfig>,
    pub ssl_config: Option<SslConfig>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInput {
    pub name: String,
    pub db_type: DbType,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub ssh_config: Option<SshConfig>,
    pub ssl_config: Option<SslConfig>,
}
```

**Step 3: Create schema models**

Create `src-tauri/src/models/schema.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub auto_increment: bool,
    pub comment: Option<String>,
    pub ordinal_position: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrimaryKey {
    pub name: Option<String>,
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Index {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
    pub index_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ForeignKey {
    pub name: String,
    pub columns: Vec<String>,
    pub ref_table: String,
    pub ref_columns: Vec<String>,
    pub on_delete: String,
    pub on_update: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UniqueConstraint {
    pub name: String,
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<Column>,
    pub primary_key: Option<PrimaryKey>,
    pub indexes: Vec<Index>,
    pub foreign_keys: Vec<ForeignKey>,
    pub unique_constraints: Vec<UniqueConstraint>,
}
```

**Step 4: Create diff models**

Create `src-tauri/src/models/diff.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DiffType {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffItem {
    pub id: String,
    pub diff_type: DiffType,
    pub table_name: String,
    pub object_name: Option<String>,
    pub source_def: Option<String>,
    pub target_def: Option<String>,
    pub sql: String,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub items: Vec<DiffItem>,
    pub source_tables: usize,
    pub target_tables: usize,
}
```

**Step 5: Create models mod.rs**

Create `src-tauri/src/models/mod.rs`:

```rust
pub mod connection;
pub mod diff;
pub mod schema;

pub use connection::*;
pub use diff::*;
pub use schema::*;
```

**Step 6: Update lib.rs to include models**

Add to `src-tauri/src/lib.rs`:

```rust
pub mod models;
```

**Step 7: Build to verify**

Run:
```bash
cd src-tauri && cargo check
```

Expected: Build succeeds

**Step 8: Commit**

```bash
git add src-tauri/src/models/ src-tauri/src/lib.rs
git commit -m "feat: add rust data models for connection, schema, and diff"
```

---

## Phase 3: Rust Backend - Storage Layer

### Task 3.1: Create SQLite Storage for Connections

**Files:**
- Create: `src-tauri/src/storage/mod.rs`
- Create: `src-tauri/src/storage/config.rs`
- Create: `src-tauri/src/storage/crypto.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Create storage directory**

Run:
```bash
mkdir -p src-tauri/src/storage
```

**Step 2: Create crypto module**

Create `src-tauri/src/storage/crypto.rs`:

```rust
use anyhow::Result;
use keyring::Entry;

const SERVICE_NAME: &str = "database-structure-sync";

pub fn store_password(connection_id: &str, password: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, connection_id)?;
    entry.set_password(password)?;
    Ok(())
}

pub fn get_password(connection_id: &str) -> Result<String> {
    let entry = Entry::new(SERVICE_NAME, connection_id)?;
    let password = entry.get_password()?;
    Ok(password)
}

pub fn delete_password(connection_id: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, connection_id)?;
    entry.delete_credential()?;
    Ok(())
}
```

**Step 3: Create config storage module**

Create `src-tauri/src/storage/config.rs`:

```rust
use anyhow::Result;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::path::PathBuf;

use crate::models::{Connection, ConnectionInput, DbType, SshAuthMethod, SshConfig, SslConfig};
use crate::storage::crypto;

pub struct ConfigStore {
    pool: Pool<Sqlite>,
}

impl ConfigStore {
    pub async fn new(app_data_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&app_data_dir)?;
        let db_path = app_data_dir.join("config.db");
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS connections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                db_type TEXT NOT NULL,
                host TEXT NOT NULL,
                port INTEGER NOT NULL,
                username TEXT NOT NULL,
                database_name TEXT NOT NULL,
                ssh_enabled INTEGER DEFAULT 0,
                ssh_host TEXT,
                ssh_port INTEGER,
                ssh_username TEXT,
                ssh_auth_method TEXT,
                ssh_private_key_path TEXT,
                ssl_enabled INTEGER DEFAULT 0,
                ssl_ca_cert_path TEXT,
                ssl_client_cert_path TEXT,
                ssl_client_key_path TEXT,
                ssl_verify_server INTEGER DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    pub async fn list_connections(&self) -> Result<Vec<Connection>> {
        let rows = sqlx::query_as::<_, ConnectionRow>(
            "SELECT * FROM connections ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut connections = Vec::new();
        for row in rows {
            let password = crypto::get_password(&row.id).unwrap_or_default();
            let ssh_password = if row.ssh_enabled == 1 && row.ssh_auth_method.as_deref() == Some("password") {
                crypto::get_password(&format!("{}_ssh", row.id)).ok()
            } else {
                None
            };
            let ssh_passphrase = if row.ssh_enabled == 1 && row.ssh_auth_method.as_deref() == Some("privatekey") {
                crypto::get_password(&format!("{}_ssh_passphrase", row.id)).ok()
            } else {
                None
            };
            connections.push(row.into_connection(password, ssh_password, ssh_passphrase));
        }

        Ok(connections)
    }

    pub async fn get_connection(&self, id: &str) -> Result<Option<Connection>> {
        let row = sqlx::query_as::<_, ConnectionRow>(
            "SELECT * FROM connections WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let password = crypto::get_password(&row.id).unwrap_or_default();
                let ssh_password = if row.ssh_enabled == 1 && row.ssh_auth_method.as_deref() == Some("password") {
                    crypto::get_password(&format!("{}_ssh", row.id)).ok()
                } else {
                    None
                };
                let ssh_passphrase = if row.ssh_enabled == 1 && row.ssh_auth_method.as_deref() == Some("privatekey") {
                    crypto::get_password(&format!("{}_ssh_passphrase", row.id)).ok()
                } else {
                    None
                };
                Ok(Some(row.into_connection(password, ssh_password, ssh_passphrase)))
            }
            None => Ok(None),
        }
    }

    pub async fn save_connection(&self, input: ConnectionInput) -> Result<Connection> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let db_type_str = match input.db_type {
            DbType::MySQL => "mysql",
            DbType::PostgreSQL => "postgresql",
            DbType::MariaDB => "mariadb",
        };

        let (ssh_enabled, ssh_host, ssh_port, ssh_username, ssh_auth_method, ssh_private_key_path) =
            match &input.ssh_config {
                Some(ssh) if ssh.enabled => {
                    let (method, key_path) = match &ssh.auth_method {
                        SshAuthMethod::Password { password } => {
                            crypto::store_password(&format!("{}_ssh", id), password)?;
                            ("password".to_string(), None)
                        }
                        SshAuthMethod::PrivateKey { private_key_path, passphrase } => {
                            if let Some(pp) = passphrase {
                                crypto::store_password(&format!("{}_ssh_passphrase", id), pp)?;
                            }
                            ("privatekey".to_string(), Some(private_key_path.clone()))
                        }
                    };
                    (1, Some(ssh.host.clone()), Some(ssh.port as i32), Some(ssh.username.clone()), Some(method), key_path)
                }
                _ => (0, None, None, None, None, None),
            };

        let (ssl_enabled, ssl_ca, ssl_cert, ssl_key, ssl_verify) = match &input.ssl_config {
            Some(ssl) if ssl.enabled => (
                1,
                ssl.ca_cert_path.clone(),
                ssl.client_cert_path.clone(),
                ssl.client_key_path.clone(),
                if ssl.verify_server { 1 } else { 0 },
            ),
            _ => (0, None, None, None, 1),
        };

        crypto::store_password(&id, &input.password)?;

        sqlx::query(
            r#"
            INSERT INTO connections (
                id, name, db_type, host, port, username, database_name,
                ssh_enabled, ssh_host, ssh_port, ssh_username, ssh_auth_method, ssh_private_key_path,
                ssl_enabled, ssl_ca_cert_path, ssl_client_cert_path, ssl_client_key_path, ssl_verify_server,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&input.name)
        .bind(db_type_str)
        .bind(&input.host)
        .bind(input.port as i32)
        .bind(&input.username)
        .bind(&input.database)
        .bind(ssh_enabled)
        .bind(&ssh_host)
        .bind(ssh_port)
        .bind(&ssh_username)
        .bind(&ssh_auth_method)
        .bind(&ssh_private_key_path)
        .bind(ssl_enabled)
        .bind(&ssl_ca)
        .bind(&ssl_cert)
        .bind(&ssl_key)
        .bind(ssl_verify)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(Connection {
            id,
            name: input.name,
            db_type: input.db_type,
            host: input.host,
            port: input.port,
            username: input.username,
            password: input.password,
            database: input.database,
            ssh_config: input.ssh_config,
            ssl_config: input.ssl_config,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub async fn delete_connection(&self, id: &str) -> Result<()> {
        let _ = crypto::delete_password(id);
        let _ = crypto::delete_password(&format!("{}_ssh", id));
        let _ = crypto::delete_password(&format!("{}_ssh_passphrase", id));

        sqlx::query("DELETE FROM connections WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct ConnectionRow {
    id: String,
    name: String,
    db_type: String,
    host: String,
    port: i32,
    username: String,
    database_name: String,
    ssh_enabled: i32,
    ssh_host: Option<String>,
    ssh_port: Option<i32>,
    ssh_username: Option<String>,
    ssh_auth_method: Option<String>,
    ssh_private_key_path: Option<String>,
    ssl_enabled: i32,
    ssl_ca_cert_path: Option<String>,
    ssl_client_cert_path: Option<String>,
    ssl_client_key_path: Option<String>,
    ssl_verify_server: i32,
    created_at: String,
    updated_at: String,
}

impl ConnectionRow {
    fn into_connection(self, password: String, ssh_password: Option<String>, ssh_passphrase: Option<String>) -> Connection {
        let db_type = match self.db_type.as_str() {
            "mysql" => DbType::MySQL,
            "postgresql" => DbType::PostgreSQL,
            "mariadb" => DbType::MariaDB,
            _ => DbType::MySQL,
        };

        let ssh_config = if self.ssh_enabled == 1 {
            let auth_method = match self.ssh_auth_method.as_deref() {
                Some("password") => SshAuthMethod::Password {
                    password: ssh_password.unwrap_or_default(),
                },
                Some("privatekey") => SshAuthMethod::PrivateKey {
                    private_key_path: self.ssh_private_key_path.unwrap_or_default(),
                    passphrase: ssh_passphrase,
                },
                _ => SshAuthMethod::Password { password: String::new() },
            };
            Some(SshConfig {
                enabled: true,
                host: self.ssh_host.unwrap_or_default(),
                port: self.ssh_port.unwrap_or(22) as u16,
                username: self.ssh_username.unwrap_or_default(),
                auth_method,
            })
        } else {
            None
        };

        let ssl_config = if self.ssl_enabled == 1 {
            Some(SslConfig {
                enabled: true,
                ca_cert_path: self.ssl_ca_cert_path,
                client_cert_path: self.ssl_client_cert_path,
                client_key_path: self.ssl_client_key_path,
                verify_server: self.ssl_verify_server == 1,
            })
        } else {
            None
        };

        Connection {
            id: self.id,
            name: self.name,
            db_type,
            host: self.host,
            port: self.port as u16,
            username: self.username,
            password,
            database: self.database_name,
            ssh_config,
            ssl_config,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
```

**Step 4: Create storage mod.rs**

Create `src-tauri/src/storage/mod.rs`:

```rust
pub mod config;
pub mod crypto;

pub use config::ConfigStore;
```

**Step 5: Update lib.rs**

Add to `src-tauri/src/lib.rs`:

```rust
pub mod storage;
```

**Step 6: Build to verify**

Run:
```bash
cd src-tauri && cargo check
```

Expected: Build succeeds

**Step 7: Commit**

```bash
git add src-tauri/src/storage/ src-tauri/src/lib.rs
git commit -m "feat: add sqlite storage layer with keychain password encryption"
```

---

## Phase 4: Rust Backend - Database Drivers

### Task 4.1: Create Database Abstraction Trait

**Files:**
- Create: `src-tauri/src/db/mod.rs`
- Create: `src-tauri/src/db/traits.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Create db directory**

Run:
```bash
mkdir -p src-tauri/src/db
```

**Step 2: Create database traits**

Create `src-tauri/src/db/traits.rs`:

```rust
use anyhow::Result;
use async_trait::async_trait;

use crate::models::{Column, Index, TableSchema};

#[async_trait]
pub trait SchemaReader: Send + Sync {
    async fn test_connection(&self) -> Result<()>;
    async fn get_tables(&self) -> Result<Vec<TableSchema>>;
}

pub trait SqlGenerator: Send + Sync {
    fn quote_identifier(&self, name: &str) -> String;
    fn generate_create_table(&self, table: &TableSchema) -> String;
    fn generate_drop_table(&self, table_name: &str) -> String;
    fn generate_add_column(&self, table: &str, column: &Column) -> String;
    fn generate_drop_column(&self, table: &str, column_name: &str) -> String;
    fn generate_modify_column(&self, table: &str, column: &Column) -> String;
    fn generate_add_index(&self, table: &str, index: &Index) -> String;
    fn generate_drop_index(&self, table: &str, index_name: &str) -> String;
    fn generate_add_foreign_key(&self, table: &str, fk: &crate::models::ForeignKey) -> String;
    fn generate_drop_foreign_key(&self, table: &str, fk_name: &str) -> String;
    fn generate_add_unique(&self, table: &str, uc: &crate::models::UniqueConstraint) -> String;
    fn generate_drop_unique(&self, table: &str, uc_name: &str) -> String;
}
```

**Step 3: Create db mod.rs**

Create `src-tauri/src/db/mod.rs`:

```rust
pub mod traits;

pub use traits::{SchemaReader, SqlGenerator};
```

**Step 4: Update lib.rs**

Add to `src-tauri/src/lib.rs`:

```rust
pub mod db;
```

**Step 5: Build to verify**

Run:
```bash
cd src-tauri && cargo check
```

**Step 6: Commit**

```bash
git add src-tauri/src/db/ src-tauri/src/lib.rs
git commit -m "feat: add database abstraction traits for schema reading and sql generation"
```

---

### Task 4.2: Implement MySQL/MariaDB Driver

**Files:**
- Create: `src-tauri/src/db/mysql.rs`
- Modify: `src-tauri/src/db/mod.rs`

**Step 1: Create MySQL driver**

Create `src-tauri/src/db/mysql.rs`:

```rust
use anyhow::Result;
use async_trait::async_trait;
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};

use crate::db::traits::{SchemaReader, SqlGenerator};
use crate::models::*;

pub struct MySqlDriver {
    pool: Pool<MySql>,
}

impl MySqlDriver {
    pub async fn new(host: &str, port: u16, user: &str, password: &str, database: &str) -> Result<Self> {
        let url = format!(
            "mysql://{}:{}@{}:{}/{}",
            user, password, host, port, database
        );
        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl SchemaReader for MySqlDriver {
    async fn test_connection(&self) -> Result<()> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }

    async fn get_tables(&self) -> Result<Vec<TableSchema>> {
        let table_names: Vec<(String,)> = sqlx::query_as(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = DATABASE() AND table_type = 'BASE TABLE'"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut tables = Vec::new();
        for (table_name,) in table_names {
            let columns = self.get_columns(&table_name).await?;
            let primary_key = self.get_primary_key(&table_name).await?;
            let indexes = self.get_indexes(&table_name).await?;
            let foreign_keys = self.get_foreign_keys(&table_name).await?;
            let unique_constraints = self.get_unique_constraints(&table_name).await?;

            tables.push(TableSchema {
                name: table_name,
                columns,
                primary_key,
                indexes,
                foreign_keys,
                unique_constraints,
            });
        }

        Ok(tables)
    }
}

impl MySqlDriver {
    async fn get_columns(&self, table_name: &str) -> Result<Vec<Column>> {
        let rows: Vec<(String, String, String, Option<String>, String, Option<String>, u32)> = sqlx::query_as(
            r#"
            SELECT
                column_name,
                column_type,
                is_nullable,
                column_default,
                extra,
                column_comment,
                ordinal_position
            FROM information_schema.columns
            WHERE table_schema = DATABASE() AND table_name = ?
            ORDER BY ordinal_position
            "#
        )
        .bind(table_name)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(name, data_type, nullable, default, extra, comment, pos)| {
            Column {
                name,
                data_type,
                nullable: nullable == "YES",
                default_value: default,
                auto_increment: extra.contains("auto_increment"),
                comment: if comment.as_ref().map(|c| c.is_empty()).unwrap_or(true) { None } else { comment },
                ordinal_position: pos,
            }
        }).collect())
    }

    async fn get_primary_key(&self, table_name: &str) -> Result<Option<PrimaryKey>> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            r#"
            SELECT constraint_name, column_name
            FROM information_schema.key_column_usage
            WHERE table_schema = DATABASE() AND table_name = ? AND constraint_name = 'PRIMARY'
            ORDER BY ordinal_position
            "#
        )
        .bind(table_name)
        .fetch_all(&self.pool)
        .await?;

        if rows.is_empty() {
            return Ok(None);
        }

        let columns: Vec<String> = rows.into_iter().map(|(_, col)| col).collect();
        Ok(Some(PrimaryKey { name: Some("PRIMARY".to_string()), columns }))
    }

    async fn get_indexes(&self, table_name: &str) -> Result<Vec<Index>> {
        let rows: Vec<(String, i32, String, String)> = sqlx::query_as(
            r#"
            SELECT index_name, non_unique, column_name, index_type
            FROM information_schema.statistics
            WHERE table_schema = DATABASE() AND table_name = ? AND index_name != 'PRIMARY'
            ORDER BY index_name, seq_in_index
            "#
        )
        .bind(table_name)
        .fetch_all(&self.pool)
        .await?;

        let mut indexes_map: std::collections::HashMap<String, (bool, String, Vec<String>)> = std::collections::HashMap::new();
        for (name, non_unique, column, idx_type) in rows {
            let entry = indexes_map.entry(name).or_insert((non_unique == 0, idx_type, Vec::new()));
            entry.2.push(column);
        }

        Ok(indexes_map.into_iter().map(|(name, (unique, idx_type, columns))| {
            Index { name, columns, unique, index_type: idx_type }
        }).collect())
    }

    async fn get_foreign_keys(&self, table_name: &str) -> Result<Vec<ForeignKey>> {
        let rows: Vec<(String, String, String, String, String, String)> = sqlx::query_as(
            r#"
            SELECT
                kcu.constraint_name,
                kcu.column_name,
                kcu.referenced_table_name,
                kcu.referenced_column_name,
                rc.delete_rule,
                rc.update_rule
            FROM information_schema.key_column_usage kcu
            JOIN information_schema.referential_constraints rc
                ON kcu.constraint_name = rc.constraint_name AND kcu.table_schema = rc.constraint_schema
            WHERE kcu.table_schema = DATABASE() AND kcu.table_name = ? AND kcu.referenced_table_name IS NOT NULL
            ORDER BY kcu.constraint_name, kcu.ordinal_position
            "#
        )
        .bind(table_name)
        .fetch_all(&self.pool)
        .await?;

        let mut fks_map: std::collections::HashMap<String, (String, String, Vec<String>, Vec<String>, String, String)> = std::collections::HashMap::new();
        for (name, col, ref_table, ref_col, on_delete, on_update) in rows {
            let entry = fks_map.entry(name.clone()).or_insert((name, ref_table, Vec::new(), Vec::new(), on_delete, on_update));
            entry.2.push(col);
            entry.3.push(ref_col);
        }

        Ok(fks_map.into_iter().map(|(_, (name, ref_table, columns, ref_columns, on_delete, on_update))| {
            ForeignKey { name, columns, ref_table, ref_columns, on_delete, on_update }
        }).collect())
    }

    async fn get_unique_constraints(&self, table_name: &str) -> Result<Vec<UniqueConstraint>> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            r#"
            SELECT tc.constraint_name, kcu.column_name
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu
                ON tc.constraint_name = kcu.constraint_name AND tc.table_schema = kcu.table_schema
            WHERE tc.table_schema = DATABASE() AND tc.table_name = ? AND tc.constraint_type = 'UNIQUE'
            ORDER BY tc.constraint_name, kcu.ordinal_position
            "#
        )
        .bind(table_name)
        .fetch_all(&self.pool)
        .await?;

        let mut ucs_map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
        for (name, col) in rows {
            ucs_map.entry(name).or_default().push(col);
        }

        Ok(ucs_map.into_iter().map(|(name, columns)| {
            UniqueConstraint { name, columns }
        }).collect())
    }
}

impl SqlGenerator for MySqlDriver {
    fn quote_identifier(&self, name: &str) -> String {
        format!("`{}`", name.replace('`', "``"))
    }

    fn generate_create_table(&self, table: &TableSchema) -> String {
        let mut sql = format!("CREATE TABLE {} (\n", self.quote_identifier(&table.name));

        let mut parts: Vec<String> = Vec::new();

        for col in &table.columns {
            let mut col_def = format!("  {} {}", self.quote_identifier(&col.name), col.data_type);
            if !col.nullable {
                col_def.push_str(" NOT NULL");
            }
            if let Some(default) = &col.default_value {
                col_def.push_str(&format!(" DEFAULT {}", default));
            }
            if col.auto_increment {
                col_def.push_str(" AUTO_INCREMENT");
            }
            if let Some(comment) = &col.comment {
                col_def.push_str(&format!(" COMMENT '{}'", comment.replace('\'', "''")));
            }
            parts.push(col_def);
        }

        if let Some(pk) = &table.primary_key {
            let cols: Vec<String> = pk.columns.iter().map(|c| self.quote_identifier(c)).collect();
            parts.push(format!("  PRIMARY KEY ({})", cols.join(", ")));
        }

        for idx in &table.indexes {
            let cols: Vec<String> = idx.columns.iter().map(|c| self.quote_identifier(c)).collect();
            let idx_type = if idx.unique { "UNIQUE INDEX" } else { "INDEX" };
            parts.push(format!("  {} {} ({})", idx_type, self.quote_identifier(&idx.name), cols.join(", ")));
        }

        for uc in &table.unique_constraints {
            let cols: Vec<String> = uc.columns.iter().map(|c| self.quote_identifier(c)).collect();
            parts.push(format!("  CONSTRAINT {} UNIQUE ({})", self.quote_identifier(&uc.name), cols.join(", ")));
        }

        for fk in &table.foreign_keys {
            let cols: Vec<String> = fk.columns.iter().map(|c| self.quote_identifier(c)).collect();
            let ref_cols: Vec<String> = fk.ref_columns.iter().map(|c| self.quote_identifier(c)).collect();
            parts.push(format!(
                "  CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} ({}) ON DELETE {} ON UPDATE {}",
                self.quote_identifier(&fk.name),
                cols.join(", "),
                self.quote_identifier(&fk.ref_table),
                ref_cols.join(", "),
                fk.on_delete,
                fk.on_update
            ));
        }

        sql.push_str(&parts.join(",\n"));
        sql.push_str("\n);");
        sql
    }

    fn generate_drop_table(&self, table_name: &str) -> String {
        format!("DROP TABLE {};", self.quote_identifier(table_name))
    }

    fn generate_add_column(&self, table: &str, column: &Column) -> String {
        let mut sql = format!(
            "ALTER TABLE {} ADD COLUMN {} {}",
            self.quote_identifier(table),
            self.quote_identifier(&column.name),
            column.data_type
        );
        if !column.nullable {
            sql.push_str(" NOT NULL");
        }
        if let Some(default) = &column.default_value {
            sql.push_str(&format!(" DEFAULT {}", default));
        }
        if column.auto_increment {
            sql.push_str(" AUTO_INCREMENT");
        }
        if let Some(comment) = &column.comment {
            sql.push_str(&format!(" COMMENT '{}'", comment.replace('\'', "''")));
        }
        sql.push(';');
        sql
    }

    fn generate_drop_column(&self, table: &str, column_name: &str) -> String {
        format!(
            "ALTER TABLE {} DROP COLUMN {};",
            self.quote_identifier(table),
            self.quote_identifier(column_name)
        )
    }

    fn generate_modify_column(&self, table: &str, column: &Column) -> String {
        let mut sql = format!(
            "ALTER TABLE {} MODIFY COLUMN {} {}",
            self.quote_identifier(table),
            self.quote_identifier(&column.name),
            column.data_type
        );
        if !column.nullable {
            sql.push_str(" NOT NULL");
        }
        if let Some(default) = &column.default_value {
            sql.push_str(&format!(" DEFAULT {}", default));
        }
        if column.auto_increment {
            sql.push_str(" AUTO_INCREMENT");
        }
        if let Some(comment) = &column.comment {
            sql.push_str(&format!(" COMMENT '{}'", comment.replace('\'', "''")));
        }
        sql.push(';');
        sql
    }

    fn generate_add_index(&self, table: &str, index: &Index) -> String {
        let cols: Vec<String> = index.columns.iter().map(|c| self.quote_identifier(c)).collect();
        let idx_type = if index.unique { "UNIQUE INDEX" } else { "INDEX" };
        format!(
            "CREATE {} {} ON {} ({});",
            idx_type,
            self.quote_identifier(&index.name),
            self.quote_identifier(table),
            cols.join(", ")
        )
    }

    fn generate_drop_index(&self, table: &str, index_name: &str) -> String {
        format!(
            "DROP INDEX {} ON {};",
            self.quote_identifier(index_name),
            self.quote_identifier(table)
        )
    }

    fn generate_add_foreign_key(&self, table: &str, fk: &ForeignKey) -> String {
        let cols: Vec<String> = fk.columns.iter().map(|c| self.quote_identifier(c)).collect();
        let ref_cols: Vec<String> = fk.ref_columns.iter().map(|c| self.quote_identifier(c)).collect();
        format!(
            "ALTER TABLE {} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} ({}) ON DELETE {} ON UPDATE {};",
            self.quote_identifier(table),
            self.quote_identifier(&fk.name),
            cols.join(", "),
            self.quote_identifier(&fk.ref_table),
            ref_cols.join(", "),
            fk.on_delete,
            fk.on_update
        )
    }

    fn generate_drop_foreign_key(&self, table: &str, fk_name: &str) -> String {
        format!(
            "ALTER TABLE {} DROP FOREIGN KEY {};",
            self.quote_identifier(table),
            self.quote_identifier(fk_name)
        )
    }

    fn generate_add_unique(&self, table: &str, uc: &UniqueConstraint) -> String {
        let cols: Vec<String> = uc.columns.iter().map(|c| self.quote_identifier(c)).collect();
        format!(
            "ALTER TABLE {} ADD CONSTRAINT {} UNIQUE ({});",
            self.quote_identifier(table),
            self.quote_identifier(&uc.name),
            cols.join(", ")
        )
    }

    fn generate_drop_unique(&self, table: &str, uc_name: &str) -> String {
        format!(
            "ALTER TABLE {} DROP INDEX {};",
            self.quote_identifier(table),
            self.quote_identifier(uc_name)
        )
    }
}
```

**Step 2: Update db mod.rs**

Add to `src-tauri/src/db/mod.rs`:

```rust
pub mod mysql;

pub use mysql::MySqlDriver;
```

**Step 3: Build to verify**

Run:
```bash
cd src-tauri && cargo check
```

**Step 4: Commit**

```bash
git add src-tauri/src/db/
git commit -m "feat: implement mysql/mariadb driver with schema reader and sql generator"
```

---

---

### Task 4.3: Implement PostgreSQL Driver

**Files:**
- Create: `src-tauri/src/db/postgres.rs`
- Modify: `src-tauri/src/db/mod.rs`

**Step 1: Create PostgreSQL driver**

Create `src-tauri/src/db/postgres.rs`:

```rust
use anyhow::Result;
use async_trait::async_trait;
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::db::traits::{SchemaReader, SqlGenerator};
use crate::models::*;

pub struct PostgresDriver {
    pool: PgPool,
}

impl PostgresDriver {
    pub async fn new(host: &str, port: u16, user: &str, password: &str, database: &str) -> Result<Self> {
        let url = format!(
            "postgres://{}:{}@{}:{}/{}",
            user, password, host, port, database
        );
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl SchemaReader for PostgresDriver {
    async fn test_connection(&self) -> Result<()> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }

    async fn get_tables(&self) -> Result<Vec<TableSchema>> {
        let table_names: Vec<(String,)> = sqlx::query_as(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE'"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut tables = Vec::new();
        for (table_name,) in table_names {
            let columns = self.get_columns(&table_name).await?;
            let primary_key = self.get_primary_key(&table_name).await?;
            let indexes = self.get_indexes(&table_name).await?;
            let foreign_keys = self.get_foreign_keys(&table_name).await?;
            let unique_constraints = self.get_unique_constraints(&table_name).await?;

            tables.push(TableSchema {
                name: table_name,
                columns,
                primary_key,
                indexes,
                foreign_keys,
                unique_constraints,
            });
        }

        Ok(tables)
    }
}

impl PostgresDriver {
    async fn get_columns(&self, table_name: &str) -> Result<Vec<Column>> {
        let rows: Vec<(String, String, String, Option<String>, i32)> = sqlx::query_as(
            r#"
            SELECT
                column_name,
                CASE
                    WHEN data_type = 'character varying' THEN 'varchar(' || character_maximum_length || ')'
                    WHEN data_type = 'character' THEN 'char(' || character_maximum_length || ')'
                    WHEN data_type = 'numeric' THEN 'numeric(' || numeric_precision || ',' || numeric_scale || ')'
                    ELSE data_type
                END as data_type,
                is_nullable,
                column_default,
                ordinal_position
            FROM information_schema.columns
            WHERE table_schema = 'public' AND table_name = $1
            ORDER BY ordinal_position
            "#
        )
        .bind(table_name)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(name, data_type, nullable, default, pos)| {
            let auto_increment = default.as_ref().map(|d| d.starts_with("nextval(")).unwrap_or(false);
            Column {
                name,
                data_type,
                nullable: nullable == "YES",
                default_value: if auto_increment { None } else { default },
                auto_increment,
                comment: None,
                ordinal_position: pos as u32,
            }
        }).collect())
    }

    async fn get_primary_key(&self, table_name: &str) -> Result<Option<PrimaryKey>> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            r#"
            SELECT tc.constraint_name, kcu.column_name
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu
                ON tc.constraint_name = kcu.constraint_name AND tc.table_schema = kcu.table_schema
            WHERE tc.table_schema = 'public' AND tc.table_name = $1 AND tc.constraint_type = 'PRIMARY KEY'
            ORDER BY kcu.ordinal_position
            "#
        )
        .bind(table_name)
        .fetch_all(&self.pool)
        .await?;

        if rows.is_empty() {
            return Ok(None);
        }

        let name = rows.first().map(|(n, _)| n.clone());
        let columns: Vec<String> = rows.into_iter().map(|(_, col)| col).collect();
        Ok(Some(PrimaryKey { name, columns }))
    }

    async fn get_indexes(&self, table_name: &str) -> Result<Vec<Index>> {
        let rows: Vec<(String, bool, String, String)> = sqlx::query_as(
            r#"
            SELECT
                i.relname as index_name,
                ix.indisunique as is_unique,
                a.attname as column_name,
                am.amname as index_type
            FROM pg_index ix
            JOIN pg_class t ON t.oid = ix.indrelid
            JOIN pg_class i ON i.oid = ix.indexrelid
            JOIN pg_am am ON i.relam = am.oid
            JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = ANY(ix.indkey)
            WHERE t.relname = $1 AND t.relnamespace = 'public'::regnamespace
                AND NOT ix.indisprimary
            ORDER BY i.relname, array_position(ix.indkey, a.attnum)
            "#
        )
        .bind(table_name)
        .fetch_all(&self.pool)
        .await?;

        let mut indexes_map: std::collections::HashMap<String, (bool, String, Vec<String>)> = std::collections::HashMap::new();
        for (name, unique, column, idx_type) in rows {
            let entry = indexes_map.entry(name).or_insert((unique, idx_type, Vec::new()));
            entry.2.push(column);
        }

        Ok(indexes_map.into_iter().map(|(name, (unique, idx_type, columns))| {
            Index { name, columns, unique, index_type: idx_type }
        }).collect())
    }

    async fn get_foreign_keys(&self, table_name: &str) -> Result<Vec<ForeignKey>> {
        let rows: Vec<(String, String, String, String, String, String)> = sqlx::query_as(
            r#"
            SELECT
                tc.constraint_name,
                kcu.column_name,
                ccu.table_name AS ref_table,
                ccu.column_name AS ref_column,
                rc.delete_rule,
                rc.update_rule
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name
            JOIN information_schema.constraint_column_usage ccu ON tc.constraint_name = ccu.constraint_name
            JOIN information_schema.referential_constraints rc ON tc.constraint_name = rc.constraint_name
            WHERE tc.table_schema = 'public' AND tc.table_name = $1 AND tc.constraint_type = 'FOREIGN KEY'
            ORDER BY tc.constraint_name, kcu.ordinal_position
            "#
        )
        .bind(table_name)
        .fetch_all(&self.pool)
        .await?;

        let mut fks_map: std::collections::HashMap<String, (String, Vec<String>, Vec<String>, String, String)> = std::collections::HashMap::new();
        for (name, col, ref_table, ref_col, on_delete, on_update) in rows {
            let entry = fks_map.entry(name).or_insert((ref_table, Vec::new(), Vec::new(), on_delete, on_update));
            entry.1.push(col);
            entry.2.push(ref_col);
        }

        Ok(fks_map.into_iter().map(|(name, (ref_table, columns, ref_columns, on_delete, on_update))| {
            ForeignKey { name, columns, ref_table, ref_columns, on_delete, on_update }
        }).collect())
    }

    async fn get_unique_constraints(&self, table_name: &str) -> Result<Vec<UniqueConstraint>> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            r#"
            SELECT tc.constraint_name, kcu.column_name
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name
            WHERE tc.table_schema = 'public' AND tc.table_name = $1 AND tc.constraint_type = 'UNIQUE'
            ORDER BY tc.constraint_name, kcu.ordinal_position
            "#
        )
        .bind(table_name)
        .fetch_all(&self.pool)
        .await?;

        let mut ucs_map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
        for (name, col) in rows {
            ucs_map.entry(name).or_default().push(col);
        }

        Ok(ucs_map.into_iter().map(|(name, columns)| {
            UniqueConstraint { name, columns }
        }).collect())
    }
}

impl SqlGenerator for PostgresDriver {
    fn quote_identifier(&self, name: &str) -> String {
        format!("\"{}\"", name.replace('"', "\"\""))
    }

    fn generate_create_table(&self, table: &TableSchema) -> String {
        let mut sql = format!("CREATE TABLE {} (\n", self.quote_identifier(&table.name));
        let mut parts: Vec<String> = Vec::new();

        for col in &table.columns {
            let data_type = if col.auto_increment { "SERIAL".to_string() } else { col.data_type.clone() };
            let mut col_def = format!("  {} {}", self.quote_identifier(&col.name), data_type);
            if !col.nullable && !col.auto_increment {
                col_def.push_str(" NOT NULL");
            }
            if let Some(default) = &col.default_value {
                col_def.push_str(&format!(" DEFAULT {}", default));
            }
            parts.push(col_def);
        }

        if let Some(pk) = &table.primary_key {
            let cols: Vec<String> = pk.columns.iter().map(|c| self.quote_identifier(c)).collect();
            parts.push(format!("  PRIMARY KEY ({})", cols.join(", ")));
        }

        for uc in &table.unique_constraints {
            let cols: Vec<String> = uc.columns.iter().map(|c| self.quote_identifier(c)).collect();
            parts.push(format!("  CONSTRAINT {} UNIQUE ({})", self.quote_identifier(&uc.name), cols.join(", ")));
        }

        for fk in &table.foreign_keys {
            let cols: Vec<String> = fk.columns.iter().map(|c| self.quote_identifier(c)).collect();
            let ref_cols: Vec<String> = fk.ref_columns.iter().map(|c| self.quote_identifier(c)).collect();
            parts.push(format!(
                "  CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} ({}) ON DELETE {} ON UPDATE {}",
                self.quote_identifier(&fk.name), cols.join(", "),
                self.quote_identifier(&fk.ref_table), ref_cols.join(", "),
                fk.on_delete, fk.on_update
            ));
        }

        sql.push_str(&parts.join(",\n"));
        sql.push_str("\n);");

        for idx in &table.indexes {
            let cols: Vec<String> = idx.columns.iter().map(|c| self.quote_identifier(c)).collect();
            let idx_type = if idx.unique { "UNIQUE INDEX" } else { "INDEX" };
            sql.push_str(&format!(
                "\nCREATE {} {} ON {} ({});",
                idx_type, self.quote_identifier(&idx.name),
                self.quote_identifier(&table.name), cols.join(", ")
            ));
        }

        sql
    }

    fn generate_drop_table(&self, table_name: &str) -> String {
        format!("DROP TABLE {};", self.quote_identifier(table_name))
    }

    fn generate_add_column(&self, table: &str, column: &Column) -> String {
        let data_type = if column.auto_increment { "SERIAL".to_string() } else { column.data_type.clone() };
        let mut sql = format!(
            "ALTER TABLE {} ADD COLUMN {} {}",
            self.quote_identifier(table), self.quote_identifier(&column.name), data_type
        );
        if !column.nullable && !column.auto_increment {
            sql.push_str(" NOT NULL");
        }
        if let Some(default) = &column.default_value {
            sql.push_str(&format!(" DEFAULT {}", default));
        }
        sql.push(';');
        sql
    }

    fn generate_drop_column(&self, table: &str, column_name: &str) -> String {
        format!("ALTER TABLE {} DROP COLUMN {};", self.quote_identifier(table), self.quote_identifier(column_name))
    }

    fn generate_modify_column(&self, table: &str, column: &Column) -> String {
        let data_type = if column.auto_increment { "SERIAL".to_string() } else { column.data_type.clone() };
        format!(
            "ALTER TABLE {} ALTER COLUMN {} TYPE {};",
            self.quote_identifier(table), self.quote_identifier(&column.name), data_type
        )
    }

    fn generate_add_index(&self, table: &str, index: &Index) -> String {
        let cols: Vec<String> = index.columns.iter().map(|c| self.quote_identifier(c)).collect();
        let idx_type = if index.unique { "UNIQUE INDEX" } else { "INDEX" };
        format!(
            "CREATE {} {} ON {} ({});",
            idx_type, self.quote_identifier(&index.name), self.quote_identifier(table), cols.join(", ")
        )
    }

    fn generate_drop_index(&self, _table: &str, index_name: &str) -> String {
        format!("DROP INDEX {};", self.quote_identifier(index_name))
    }

    fn generate_add_foreign_key(&self, table: &str, fk: &ForeignKey) -> String {
        let cols: Vec<String> = fk.columns.iter().map(|c| self.quote_identifier(c)).collect();
        let ref_cols: Vec<String> = fk.ref_columns.iter().map(|c| self.quote_identifier(c)).collect();
        format!(
            "ALTER TABLE {} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} ({}) ON DELETE {} ON UPDATE {};",
            self.quote_identifier(table), self.quote_identifier(&fk.name),
            cols.join(", "), self.quote_identifier(&fk.ref_table), ref_cols.join(", "),
            fk.on_delete, fk.on_update
        )
    }

    fn generate_drop_foreign_key(&self, table: &str, fk_name: &str) -> String {
        format!("ALTER TABLE {} DROP CONSTRAINT {};", self.quote_identifier(table), self.quote_identifier(fk_name))
    }

    fn generate_add_unique(&self, table: &str, uc: &UniqueConstraint) -> String {
        let cols: Vec<String> = uc.columns.iter().map(|c| self.quote_identifier(c)).collect();
        format!(
            "ALTER TABLE {} ADD CONSTRAINT {} UNIQUE ({});",
            self.quote_identifier(table), self.quote_identifier(&uc.name), cols.join(", ")
        )
    }

    fn generate_drop_unique(&self, table: &str, uc_name: &str) -> String {
        format!("ALTER TABLE {} DROP CONSTRAINT {};", self.quote_identifier(table), self.quote_identifier(uc_name))
    }
}
```

**Step 2: Update db mod.rs**

Add to `src-tauri/src/db/mod.rs`:

```rust
pub mod postgres;

pub use postgres::PostgresDriver;
```

**Step 3: Build to verify**

Run:
```bash
cd src-tauri && cargo check
```

**Step 4: Commit**

```bash
git add src-tauri/src/db/
git commit -m "feat: implement postgresql driver with schema reader and sql generator"
```

---

## Phase 5: Rust Backend - Diff Engine

### Task 5.1: Implement Diff Comparator

**Files:**
- Create: `src-tauri/src/diff/mod.rs`
- Create: `src-tauri/src/diff/comparator.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Create diff directory**

Run:
```bash
mkdir -p src-tauri/src/diff
```

**Step 2: Create comparator**

Create `src-tauri/src/diff/comparator.rs`:

```rust
use crate::db::SqlGenerator;
use crate::models::*;
use std::collections::HashMap;

pub fn compare_schemas(
    source: &[TableSchema],
    target: &[TableSchema],
    sql_gen: &dyn SqlGenerator,
) -> Vec<DiffItem> {
    let mut diffs = Vec::new();
    let mut id_counter = 0;

    let source_map: HashMap<&str, &TableSchema> = source.iter().map(|t| (t.name.as_str(), t)).collect();
    let target_map: HashMap<&str, &TableSchema> = target.iter().map(|t| (t.name.as_str(), t)).collect();

    // Find added tables (in source but not in target)
    for table in source {
        if !target_map.contains_key(table.name.as_str()) {
            id_counter += 1;
            diffs.push(DiffItem {
                id: id_counter.to_string(),
                diff_type: DiffType::TableAdded,
                table_name: table.name.clone(),
                object_name: None,
                source_def: Some(format!("{} columns", table.columns.len())),
                target_def: None,
                sql: sql_gen.generate_create_table(table),
                selected: true,
            });
        }
    }

    // Find removed tables (in target but not in source)
    for table in target {
        if !source_map.contains_key(table.name.as_str()) {
            id_counter += 1;
            diffs.push(DiffItem {
                id: id_counter.to_string(),
                diff_type: DiffType::TableRemoved,
                table_name: table.name.clone(),
                object_name: None,
                source_def: None,
                target_def: Some(format!("{} columns", table.columns.len())),
                sql: sql_gen.generate_drop_table(&table.name),
                selected: true,
            });
        }
    }

    // Compare existing tables
    for source_table in source {
        if let Some(target_table) = target_map.get(source_table.name.as_str()) {
            compare_tables(source_table, target_table, sql_gen, &mut diffs, &mut id_counter);
        }
    }

    diffs
}

fn compare_tables(
    source: &TableSchema,
    target: &TableSchema,
    sql_gen: &dyn SqlGenerator,
    diffs: &mut Vec<DiffItem>,
    id_counter: &mut u32,
) {
    let source_cols: HashMap<&str, &Column> = source.columns.iter().map(|c| (c.name.as_str(), c)).collect();
    let target_cols: HashMap<&str, &Column> = target.columns.iter().map(|c| (c.name.as_str(), c)).collect();

    // Compare columns
    for col in &source.columns {
        if !target_cols.contains_key(col.name.as_str()) {
            *id_counter += 1;
            diffs.push(DiffItem {
                id: id_counter.to_string(),
                diff_type: DiffType::ColumnAdded,
                table_name: source.name.clone(),
                object_name: Some(col.name.clone()),
                source_def: Some(col.data_type.clone()),
                target_def: None,
                sql: sql_gen.generate_add_column(&source.name, col),
                selected: true,
            });
        } else if let Some(target_col) = target_cols.get(col.name.as_str()) {
            if col != *target_col {
                *id_counter += 1;
                diffs.push(DiffItem {
                    id: id_counter.to_string(),
                    diff_type: DiffType::ColumnModified,
                    table_name: source.name.clone(),
                    object_name: Some(col.name.clone()),
                    source_def: Some(col.data_type.clone()),
                    target_def: Some(target_col.data_type.clone()),
                    sql: sql_gen.generate_modify_column(&source.name, col),
                    selected: true,
                });
            }
        }
    }

    for col in &target.columns {
        if !source_cols.contains_key(col.name.as_str()) {
            *id_counter += 1;
            diffs.push(DiffItem {
                id: id_counter.to_string(),
                diff_type: DiffType::ColumnRemoved,
                table_name: source.name.clone(),
                object_name: Some(col.name.clone()),
                source_def: None,
                target_def: Some(col.data_type.clone()),
                sql: sql_gen.generate_drop_column(&source.name, &col.name),
                selected: true,
            });
        }
    }

    // Compare indexes
    let source_idx: HashMap<&str, &Index> = source.indexes.iter().map(|i| (i.name.as_str(), i)).collect();
    let target_idx: HashMap<&str, &Index> = target.indexes.iter().map(|i| (i.name.as_str(), i)).collect();

    for idx in &source.indexes {
        if !target_idx.contains_key(idx.name.as_str()) {
            *id_counter += 1;
            diffs.push(DiffItem {
                id: id_counter.to_string(),
                diff_type: DiffType::IndexAdded,
                table_name: source.name.clone(),
                object_name: Some(idx.name.clone()),
                source_def: Some(idx.columns.join(", ")),
                target_def: None,
                sql: sql_gen.generate_add_index(&source.name, idx),
                selected: true,
            });
        } else if let Some(target_index) = target_idx.get(idx.name.as_str()) {
            if idx != *target_index {
                *id_counter += 1;
                diffs.push(DiffItem {
                    id: id_counter.to_string(),
                    diff_type: DiffType::IndexModified,
                    table_name: source.name.clone(),
                    object_name: Some(idx.name.clone()),
                    source_def: Some(idx.columns.join(", ")),
                    target_def: Some(target_index.columns.join(", ")),
                    sql: format!("{}\n{}", sql_gen.generate_drop_index(&source.name, &idx.name), sql_gen.generate_add_index(&source.name, idx)),
                    selected: true,
                });
            }
        }
    }

    for idx in &target.indexes {
        if !source_idx.contains_key(idx.name.as_str()) {
            *id_counter += 1;
            diffs.push(DiffItem {
                id: id_counter.to_string(),
                diff_type: DiffType::IndexRemoved,
                table_name: source.name.clone(),
                object_name: Some(idx.name.clone()),
                source_def: None,
                target_def: Some(idx.columns.join(", ")),
                sql: sql_gen.generate_drop_index(&source.name, &idx.name),
                selected: true,
            });
        }
    }

    // Compare foreign keys
    let source_fks: HashMap<&str, &ForeignKey> = source.foreign_keys.iter().map(|f| (f.name.as_str(), f)).collect();
    let target_fks: HashMap<&str, &ForeignKey> = target.foreign_keys.iter().map(|f| (f.name.as_str(), f)).collect();

    for fk in &source.foreign_keys {
        if !target_fks.contains_key(fk.name.as_str()) {
            *id_counter += 1;
            diffs.push(DiffItem {
                id: id_counter.to_string(),
                diff_type: DiffType::ForeignKeyAdded,
                table_name: source.name.clone(),
                object_name: Some(fk.name.clone()),
                source_def: Some(format!("-> {}", fk.ref_table)),
                target_def: None,
                sql: sql_gen.generate_add_foreign_key(&source.name, fk),
                selected: true,
            });
        }
    }

    for fk in &target.foreign_keys {
        if !source_fks.contains_key(fk.name.as_str()) {
            *id_counter += 1;
            diffs.push(DiffItem {
                id: id_counter.to_string(),
                diff_type: DiffType::ForeignKeyRemoved,
                table_name: source.name.clone(),
                object_name: Some(fk.name.clone()),
                source_def: None,
                target_def: Some(format!("-> {}", fk.ref_table)),
                sql: sql_gen.generate_drop_foreign_key(&source.name, &fk.name),
                selected: true,
            });
        }
    }

    // Compare unique constraints
    let source_ucs: HashMap<&str, &UniqueConstraint> = source.unique_constraints.iter().map(|u| (u.name.as_str(), u)).collect();
    let target_ucs: HashMap<&str, &UniqueConstraint> = target.unique_constraints.iter().map(|u| (u.name.as_str(), u)).collect();

    for uc in &source.unique_constraints {
        if !target_ucs.contains_key(uc.name.as_str()) {
            *id_counter += 1;
            diffs.push(DiffItem {
                id: id_counter.to_string(),
                diff_type: DiffType::UniqueConstraintAdded,
                table_name: source.name.clone(),
                object_name: Some(uc.name.clone()),
                source_def: Some(uc.columns.join(", ")),
                target_def: None,
                sql: sql_gen.generate_add_unique(&source.name, uc),
                selected: true,
            });
        }
    }

    for uc in &target.unique_constraints {
        if !source_ucs.contains_key(uc.name.as_str()) {
            *id_counter += 1;
            diffs.push(DiffItem {
                id: id_counter.to_string(),
                diff_type: DiffType::UniqueConstraintRemoved,
                table_name: source.name.clone(),
                object_name: Some(uc.name.clone()),
                source_def: None,
                target_def: Some(uc.columns.join(", ")),
                sql: sql_gen.generate_drop_unique(&source.name, &uc.name),
                selected: true,
            });
        }
    }
}
```

**Step 3: Create diff mod.rs**

Create `src-tauri/src/diff/mod.rs`:

```rust
pub mod comparator;

pub use comparator::compare_schemas;
```

**Step 4: Update lib.rs**

Add to `src-tauri/src/lib.rs`:

```rust
pub mod diff;
```

**Step 5: Build to verify**

Run:
```bash
cd src-tauri && cargo check
```

**Step 6: Commit**

```bash
git add src-tauri/src/diff/ src-tauri/src/lib.rs
git commit -m "feat: implement diff comparator for schema comparison"
```

---

## Phase 6: Rust Backend - Tauri Commands

### Task 6.1: Create Tauri Commands

**Files:**
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/connection.rs`
- Create: `src-tauri/src/commands/schema.rs`
- Create: `src-tauri/src/commands/sync.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/main.rs`

**Step 1: Create commands directory**

Run:
```bash
mkdir -p src-tauri/src/commands
```

**Step 2: Create connection commands**

Create `src-tauri/src/commands/connection.rs`:

```rust
use tauri::State;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::models::{Connection, ConnectionInput, DbType};
use crate::storage::ConfigStore;
use crate::db::{MySqlDriver, PostgresDriver, SchemaReader};

pub struct AppState {
    pub config_store: Arc<Mutex<ConfigStore>>,
}

#[tauri::command]
pub async fn list_connections(state: State<'_, AppState>) -> Result<Vec<Connection>, String> {
    let store = state.config_store.lock().await;
    store.list_connections().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_connection(state: State<'_, AppState>, id: String) -> Result<Option<Connection>, String> {
    let store = state.config_store.lock().await;
    store.get_connection(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_connection(state: State<'_, AppState>, input: ConnectionInput) -> Result<Connection, String> {
    let store = state.config_store.lock().await;
    store.save_connection(input).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_connection(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let store = state.config_store.lock().await;
    store.delete_connection(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn test_connection(input: ConnectionInput) -> Result<(), String> {
    // TODO: Add SSH tunnel support
    let reader: Box<dyn SchemaReader> = match input.db_type {
        DbType::MySQL | DbType::MariaDB => {
            Box::new(MySqlDriver::new(&input.host, input.port, &input.username, &input.password, &input.database).await.map_err(|e| e.to_string())?)
        }
        DbType::PostgreSQL => {
            Box::new(PostgresDriver::new(&input.host, input.port, &input.username, &input.password, &input.database).await.map_err(|e| e.to_string())?)
        }
    };

    reader.test_connection().await.map_err(|e| e.to_string())
}
```

**Step 3: Create schema commands**

Create `src-tauri/src/commands/schema.rs`:

```rust
use tauri::State;

use crate::commands::connection::AppState;
use crate::db::{MySqlDriver, PostgresDriver, SchemaReader, SqlGenerator};
use crate::diff::compare_schemas;
use crate::models::{DbType, DiffResult};

#[tauri::command]
pub async fn compare_databases(
    state: State<'_, AppState>,
    source_id: String,
    target_id: String,
) -> Result<DiffResult, String> {
    let store = state.config_store.lock().await;

    let source_conn = store.get_connection(&source_id).await.map_err(|e| e.to_string())?
        .ok_or("Source connection not found")?;
    let target_conn = store.get_connection(&target_id).await.map_err(|e| e.to_string())?
        .ok_or("Target connection not found")?;

    drop(store);

    // Create source reader
    let source_reader: Box<dyn SchemaReader> = match source_conn.db_type {
        DbType::MySQL | DbType::MariaDB => {
            Box::new(MySqlDriver::new(&source_conn.host, source_conn.port, &source_conn.username, &source_conn.password, &source_conn.database).await.map_err(|e| e.to_string())?)
        }
        DbType::PostgreSQL => {
            Box::new(PostgresDriver::new(&source_conn.host, source_conn.port, &source_conn.username, &source_conn.password, &source_conn.database).await.map_err(|e| e.to_string())?)
        }
    };

    // Create target reader
    let target_reader: Box<dyn SchemaReader> = match target_conn.db_type {
        DbType::MySQL | DbType::MariaDB => {
            Box::new(MySqlDriver::new(&target_conn.host, target_conn.port, &target_conn.username, &target_conn.password, &target_conn.database).await.map_err(|e| e.to_string())?)
        }
        DbType::PostgreSQL => {
            Box::new(PostgresDriver::new(&target_conn.host, target_conn.port, &target_conn.username, &target_conn.password, &target_conn.database).await.map_err(|e| e.to_string())?)
        }
    };

    // Get schemas
    let source_tables = source_reader.get_tables().await.map_err(|e| e.to_string())?;
    let target_tables = target_reader.get_tables().await.map_err(|e| e.to_string())?;

    // Create SQL generator for target database type
    let sql_gen: Box<dyn SqlGenerator> = match target_conn.db_type {
        DbType::MySQL | DbType::MariaDB => {
            Box::new(MySqlDriver::new(&target_conn.host, target_conn.port, &target_conn.username, &target_conn.password, &target_conn.database).await.map_err(|e| e.to_string())?)
        }
        DbType::PostgreSQL => {
            Box::new(PostgresDriver::new(&target_conn.host, target_conn.port, &target_conn.username, &target_conn.password, &target_conn.database).await.map_err(|e| e.to_string())?)
        }
    };

    let items = compare_schemas(&source_tables, &target_tables, sql_gen.as_ref());

    Ok(DiffResult {
        items,
        source_tables: source_tables.len(),
        target_tables: target_tables.len(),
    })
}
```

**Step 4: Create sync commands**

Create `src-tauri/src/commands/sync.rs`:

```rust
use tauri::State;
use sqlx::Executor;

use crate::commands::connection::AppState;
use crate::db::{MySqlDriver, PostgresDriver};
use crate::models::DbType;

#[tauri::command]
pub async fn execute_sync(
    state: State<'_, AppState>,
    target_id: String,
    sql_statements: Vec<String>,
) -> Result<(), String> {
    let store = state.config_store.lock().await;
    let target_conn = store.get_connection(&target_id).await.map_err(|e| e.to_string())?
        .ok_or("Target connection not found")?;
    drop(store);

    match target_conn.db_type {
        DbType::MySQL | DbType::MariaDB => {
            let driver = MySqlDriver::new(&target_conn.host, target_conn.port, &target_conn.username, &target_conn.password, &target_conn.database).await.map_err(|e| e.to_string())?;
            for sql in sql_statements {
                sqlx::query(&sql).execute(driver.pool()).await.map_err(|e| format!("Failed to execute: {}\nError: {}", sql, e))?;
            }
        }
        DbType::PostgreSQL => {
            let driver = PostgresDriver::new(&target_conn.host, target_conn.port, &target_conn.username, &target_conn.password, &target_conn.database).await.map_err(|e| e.to_string())?;
            for sql in sql_statements {
                sqlx::query(&sql).execute(driver.pool()).await.map_err(|e| format!("Failed to execute: {}\nError: {}", sql, e))?;
            }
        }
    }

    Ok(())
}
```

**Step 5: Create commands mod.rs**

Create `src-tauri/src/commands/mod.rs`:

```rust
pub mod connection;
pub mod schema;
pub mod sync;

pub use connection::*;
pub use schema::*;
pub use sync::*;
```

**Step 6: Update MySQL driver to expose pool**

Add to `src-tauri/src/db/mysql.rs` in the `impl MySqlDriver` block:

```rust
pub fn pool(&self) -> &Pool<MySql> {
    &self.pool
}
```

**Step 7: Update PostgreSQL driver to expose pool**

Add to `src-tauri/src/db/postgres.rs` in the `impl PostgresDriver` block:

```rust
pub fn pool(&self) -> &PgPool {
    &self.pool
}
```

**Step 8: Update lib.rs**

Add to `src-tauri/src/lib.rs`:

```rust
pub mod commands;
```

**Step 9: Update main.rs**

Replace `src-tauri/src/main.rs`:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::Manager;

use database_structure_synchronization::commands::{
    AppState, list_connections, get_connection, save_connection, delete_connection,
    test_connection, compare_databases, execute_sync
};
use database_structure_synchronization::storage::ConfigStore;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("Failed to get app data dir");

            tauri::async_runtime::block_on(async {
                let config_store = ConfigStore::new(app_data_dir).await.expect("Failed to initialize config store");
                app.manage(AppState {
                    config_store: Arc::new(Mutex::new(config_store)),
                });
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_connections,
            get_connection,
            save_connection,
            delete_connection,
            test_connection,
            compare_databases,
            execute_sync
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Step 10: Build to verify**

Run:
```bash
cd src-tauri && cargo check
```

**Step 11: Commit**

```bash
git add src-tauri/src/
git commit -m "feat: implement tauri commands for connection, schema comparison, and sync"
```

---

## Phase 7: Frontend Implementation

### Task 7.1: Create Base Layout

**Files:**
- Modify: `src/App.tsx`
- Create: `src/components/layout/MainLayout.tsx`
- Create: `src/components/layout/Sidebar.tsx`

**Step 1: Create layout components directory**

Run:
```bash
mkdir -p src/components/layout
```

**Step 2: Create MainLayout component**

Create `src/components/layout/MainLayout.tsx`:

```tsx
import { ReactNode } from 'react';
import { Sidebar } from './Sidebar';

interface MainLayoutProps {
  children: ReactNode;
}

export function MainLayout({ children }: MainLayoutProps) {
  return (
    <div className="flex h-screen bg-background">
      <Sidebar />
      <main className="flex-1 overflow-auto p-6">
        {children}
      </main>
    </div>
  );
}
```

**Step 3: Create Sidebar component**

Create `src/components/layout/Sidebar.tsx`:

```tsx
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Plus, Database, ChevronDown, ChevronRight, Trash2, Edit } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';

interface Connection {
  id: string;
  name: string;
  db_type: string;
  host: string;
  database: string;
}

interface SidebarProps {
  connections?: Connection[];
  onNewConnection?: () => void;
  onEditConnection?: (id: string) => void;
  onDeleteConnection?: (id: string) => void;
  onSelectConnection?: (id: string) => void;
  selectedId?: string;
}

export function Sidebar({
  connections = [],
  onNewConnection,
  onEditConnection,
  onDeleteConnection,
  onSelectConnection,
  selectedId,
}: SidebarProps) {
  const { t, i18n } = useTranslation();
  const [expanded, setExpanded] = useState<Record<string, boolean>>({});

  const toggleExpand = (id: string) => {
    setExpanded((prev) => ({ ...prev, [id]: !prev[id] }));
  };

  const toggleLanguage = () => {
    const newLang = i18n.language === 'en' ? 'zh' : 'en';
    i18n.changeLanguage(newLang);
    localStorage.setItem('language', newLang);
  };

  return (
    <div className="w-64 border-r bg-muted/30 flex flex-col">
      <div className="p-4 border-b flex items-center justify-between">
        <h2 className="font-semibold text-sm">{t('connection.title')}</h2>
        <Button variant="ghost" size="sm" onClick={toggleLanguage}>
          {i18n.language === 'en' ? '中文' : 'EN'}
        </Button>
      </div>

      <ScrollArea className="flex-1">
        <div className="p-2">
          {connections.map((conn) => (
            <div key={conn.id} className="mb-1">
              <div
                className={`flex items-center gap-2 p-2 rounded-md cursor-pointer hover:bg-muted ${
                  selectedId === conn.id ? 'bg-muted' : ''
                }`}
                onClick={() => onSelectConnection?.(conn.id)}
              >
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    toggleExpand(conn.id);
                  }}
                  className="p-0.5"
                >
                  {expanded[conn.id] ? (
                    <ChevronDown className="h-4 w-4" />
                  ) : (
                    <ChevronRight className="h-4 w-4" />
                  )}
                </button>
                <Database className="h-4 w-4 text-muted-foreground" />
                <span className="flex-1 text-sm truncate">{conn.name}</span>
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button variant="ghost" size="icon" className="h-6 w-6">
                      <span className="sr-only">Actions</span>
                      ...
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end">
                    <DropdownMenuItem onClick={() => onEditConnection?.(conn.id)}>
                      <Edit className="h-4 w-4 mr-2" />
                      {t('connection.edit')}
                    </DropdownMenuItem>
                    <DropdownMenuItem
                      onClick={() => onDeleteConnection?.(conn.id)}
                      className="text-destructive"
                    >
                      <Trash2 className="h-4 w-4 mr-2" />
                      {t('connection.delete')}
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>
              {expanded[conn.id] && (
                <div className="ml-8 text-xs text-muted-foreground p-2">
                  <div>{conn.db_type}</div>
                  <div>{conn.host}</div>
                  <div>{conn.database}</div>
                </div>
              )}
            </div>
          ))}
        </div>
      </ScrollArea>

      <div className="p-4 border-t">
        <Button onClick={onNewConnection} className="w-full" size="sm">
          <Plus className="h-4 w-4 mr-2" />
          {t('connection.new')}
        </Button>
      </div>
    </div>
  );
}
```

**Step 4: Update App.tsx**

Replace `src/App.tsx`:

```tsx
import { MainLayout } from './components/layout/MainLayout';

function App() {
  return (
    <MainLayout>
      <div className="text-center">
        <h1 className="text-2xl font-bold">Database Structure Sync</h1>
        <p className="text-muted-foreground mt-2">Select connections to compare</p>
      </div>
    </MainLayout>
  );
}

export default App;
```

**Step 5: Run to verify**

Run:
```bash
npm run tauri dev
```

Expected: App opens with sidebar and main content area

**Step 6: Commit**

```bash
git add src/
git commit -m "feat: add base layout with sidebar component"
```

---

The implementation plan continues with additional frontend tasks (connection form, diff tree, SQL preview) and integration. Due to the size, I've provided the core structure that covers:

- Phase 1: Project Setup (Tasks 1.1-1.4)
- Phase 2: Data Models (Task 2.1)
- Phase 3: Storage Layer (Task 3.1)
- Phase 4: Database Drivers (Tasks 4.1-4.3)
- Phase 5: Diff Engine (Task 5.1)
- Phase 6: Tauri Commands (Task 6.1)
- Phase 7: Frontend Implementation (Task 7.1 - partial)

Remaining tasks to add:
- Task 7.2: Connection Form Dialog
- Task 7.3: Sync Page with Database Selectors
- Task 7.4: Diff Tree Component
- Task 7.5: SQL Preview Component
- Task 8.1: SSH Tunnel Implementation
- Task 8.2: SSL/TLS Support
- Task 9.1: Integration Testing
- Task 9.2: Build and Package
