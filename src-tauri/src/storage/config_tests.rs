use super::*;

fn base_row() -> ConnectionRow {
    ConnectionRow {
        id: "test-id".to_string(),
        name: "Test".to_string(),
        db_type: "mysql".to_string(),
        host: "localhost".to_string(),
        port: 3306,
        username: "root".to_string(),
        database_name: "testdb".to_string(),
        ssh_enabled: 0,
        ssh_host: None,
        ssh_port: None,
        ssh_username: None,
        ssh_auth_method: None,
        ssh_private_key_path: None,
        ssl_enabled: 0,
        ssl_ca_cert_path: None,
        ssl_client_cert_path: None,
        ssl_client_key_path: None,
        ssl_verify_server: 1,
        created_at: "2025-01-01".to_string(),
        updated_at: "2025-01-01".to_string(),
    }
}

// ========================================================================
// db_type mapping
// ========================================================================

#[test]
fn into_connection_mysql_db_type() {
    let row = base_row();
    let conn = row.into_connection("pw".into(), None, None);
    assert!(matches!(conn.db_type, DbType::MySQL));
}

#[test]
fn into_connection_postgresql_db_type() {
    let mut row = base_row();
    row.db_type = "postgresql".to_string();
    let conn = row.into_connection("pw".into(), None, None);
    assert!(matches!(conn.db_type, DbType::PostgreSQL));
}

#[test]
fn into_connection_mariadb_db_type() {
    let mut row = base_row();
    row.db_type = "mariadb".to_string();
    let conn = row.into_connection("pw".into(), None, None);
    assert!(matches!(conn.db_type, DbType::MariaDB));
}

#[test]
fn into_connection_unknown_db_type_falls_back_to_mysql() {
    let mut row = base_row();
    row.db_type = "oracle".to_string();
    let conn = row.into_connection("pw".into(), None, None);
    assert!(matches!(conn.db_type, DbType::MySQL));
}

#[test]
fn into_connection_empty_db_type_falls_back_to_mysql() {
    let mut row = base_row();
    row.db_type = "".to_string();
    let conn = row.into_connection("pw".into(), None, None);
    assert!(matches!(conn.db_type, DbType::MySQL));
}

// ========================================================================
// SSH config
// ========================================================================

#[test]
fn into_connection_ssh_password_auth() {
    let mut row = base_row();
    row.ssh_enabled = 1;
    row.ssh_host = Some("bastion.host".to_string());
    row.ssh_port = Some(2222);
    row.ssh_username = Some("jump".to_string());
    row.ssh_auth_method = Some("password".to_string());

    let conn = row.into_connection("pw".into(), Some("ssh_pw".into()), None);
    let ssh = conn
        .ssh_config
        .expect("ssh_config should be Some when ssh_enabled=1");
    assert!(ssh.enabled);
    assert_eq!(ssh.host, "bastion.host");
    assert_eq!(ssh.port, 2222);
    assert_eq!(ssh.username, "jump");
    match ssh.auth_method {
        SshAuthMethod::Password { password } => assert_eq!(password, "ssh_pw"),
        _ => panic!("expected Password auth"),
    }
}

#[test]
fn into_connection_ssh_privatekey_auth() {
    let mut row = base_row();
    row.ssh_enabled = 1;
    row.ssh_host = Some("bastion.host".to_string());
    row.ssh_port = Some(22);
    row.ssh_username = Some("jump".to_string());
    row.ssh_auth_method = Some("privatekey".to_string());
    row.ssh_private_key_path = Some("/home/.ssh/id_rsa".to_string());

    let conn = row.into_connection("pw".into(), None, Some("passphrase".into()));
    let ssh = conn
        .ssh_config
        .expect("ssh_config should be Some when ssh_enabled=1");
    match ssh.auth_method {
        SshAuthMethod::PrivateKey {
            private_key_path,
            passphrase,
        } => {
            assert_eq!(private_key_path, "/home/.ssh/id_rsa");
            assert_eq!(passphrase, Some("passphrase".to_string()));
        }
        _ => panic!("expected PrivateKey auth"),
    }
}

#[test]
fn into_connection_ssh_unknown_auth_method_falls_back_to_empty_password() {
    let mut row = base_row();
    row.ssh_enabled = 1;
    row.ssh_host = Some("host".to_string());
    row.ssh_username = Some("user".to_string());
    row.ssh_auth_method = Some("kerberos".to_string());

    let conn = row.into_connection("pw".into(), None, None);
    let ssh = conn
        .ssh_config
        .expect("ssh_config should be Some when ssh_enabled=1");
    match ssh.auth_method {
        SshAuthMethod::Password { password } => assert_eq!(password, ""),
        _ => panic!("expected fallback to Password with empty string"),
    }
}

#[test]
fn into_connection_ssh_none_auth_method_falls_back_to_empty_password() {
    let mut row = base_row();
    row.ssh_enabled = 1;
    row.ssh_host = Some("host".to_string());
    row.ssh_username = Some("user".to_string());
    row.ssh_auth_method = None;

    let conn = row.into_connection("pw".into(), None, None);
    let ssh = conn
        .ssh_config
        .expect("ssh_config should be Some when ssh_enabled=1");
    match ssh.auth_method {
        SshAuthMethod::Password { password } => assert_eq!(password, ""),
        _ => panic!("expected fallback to Password with empty string"),
    }
}

#[test]
fn into_connection_ssh_disabled_returns_none() {
    let row = base_row(); // ssh_enabled = 0
    let conn = row.into_connection("pw".into(), None, None);
    assert!(conn.ssh_config.is_none());
}

#[test]
fn into_connection_ssh_port_none_defaults_to_22() {
    let mut row = base_row();
    row.ssh_enabled = 1;
    row.ssh_host = Some("host".to_string());
    row.ssh_username = Some("user".to_string());
    row.ssh_port = None;
    row.ssh_auth_method = Some("password".to_string());

    let conn = row.into_connection("pw".into(), Some("sp".into()), None);
    assert_eq!(conn.ssh_config.expect("ssh_config should be Some").port, 22);
}

// ========================================================================
// SSL config
// ========================================================================

#[test]
fn into_connection_ssl_enabled() {
    let mut row = base_row();
    row.ssl_enabled = 1;
    row.ssl_ca_cert_path = Some("/certs/ca.pem".to_string());
    row.ssl_client_cert_path = Some("/certs/client.pem".to_string());
    row.ssl_client_key_path = Some("/certs/key.pem".to_string());
    row.ssl_verify_server = 1;

    let conn = row.into_connection("pw".into(), None, None);
    let ssl = conn
        .ssl_config
        .expect("ssl_config should be Some when ssl_enabled=1");
    assert!(ssl.enabled);
    assert_eq!(ssl.ca_cert_path, Some("/certs/ca.pem".to_string()));
    assert_eq!(ssl.client_cert_path, Some("/certs/client.pem".to_string()));
    assert_eq!(ssl.client_key_path, Some("/certs/key.pem".to_string()));
    assert!(ssl.verify_server);
}

#[test]
fn into_connection_ssl_disabled_returns_none() {
    let row = base_row(); // ssl_enabled = 0
    let conn = row.into_connection("pw".into(), None, None);
    assert!(conn.ssl_config.is_none());
}

#[test]
fn into_connection_ssl_verify_server_false() {
    let mut row = base_row();
    row.ssl_enabled = 1;
    row.ssl_verify_server = 0;

    let conn = row.into_connection("pw".into(), None, None);
    let ssl = conn
        .ssl_config
        .expect("ssl_config should be Some when ssl_enabled=1");
    assert!(!ssl.verify_server);
}

// ========================================================================
// Basic field mapping
// ========================================================================

#[test]
fn into_connection_maps_basic_fields() {
    let row = base_row();
    let conn = row.into_connection("secret".into(), None, None);
    assert_eq!(conn.id, "test-id");
    assert_eq!(conn.name, "Test");
    assert_eq!(conn.host, "localhost");
    assert_eq!(conn.port, 3306);
    assert_eq!(conn.username, "root");
    assert_eq!(conn.password, "secret");
    assert_eq!(conn.database, "testdb");
    assert_eq!(conn.created_at, "2025-01-01");
    assert_eq!(conn.updated_at, "2025-01-01");
}

#[test]
fn into_connection_ssh_privatekey_no_passphrase() {
    let mut row = base_row();
    row.ssh_enabled = 1;
    row.ssh_host = Some("h".to_string());
    row.ssh_username = Some("u".to_string());
    row.ssh_auth_method = Some("privatekey".to_string());
    row.ssh_private_key_path = Some("/key".to_string());

    let conn = row.into_connection("pw".into(), None, None);
    let ssh = conn
        .ssh_config
        .expect("ssh_config should be Some when ssh_enabled=1");
    match ssh.auth_method {
        SshAuthMethod::PrivateKey { passphrase, .. } => assert_eq!(passphrase, None),
        _ => panic!("expected PrivateKey"),
    }
}

#[test]
fn into_connection_ssh_missing_fields_default_to_empty() {
    let mut row = base_row();
    row.ssh_enabled = 1;
    row.ssh_host = None;
    row.ssh_username = None;
    row.ssh_auth_method = Some("password".to_string());

    let conn = row.into_connection("pw".into(), Some("sp".into()), None);
    let ssh = conn
        .ssh_config
        .expect("ssh_config should be Some when ssh_enabled=1");
    assert_eq!(ssh.host, "");
    assert_eq!(ssh.username, "");
}

#[test]
fn into_connection_ssh_password_auth_with_none_ssh_password() {
    let mut row = base_row();
    row.ssh_enabled = 1;
    row.ssh_host = Some("host".to_string());
    row.ssh_username = Some("user".to_string());
    row.ssh_auth_method = Some("password".to_string());

    // ssh_password is None — should fall back to empty string via unwrap_or_default
    let conn = row.into_connection("pw".into(), None, None);
    let ssh = conn
        .ssh_config
        .expect("ssh_config should be Some when ssh_enabled=1");
    match ssh.auth_method {
        SshAuthMethod::Password { password } => assert_eq!(password, ""),
        _ => panic!("expected Password auth with empty string"),
    }
}

#[test]
fn into_connection_ssl_verify_server_nonzero_nonone_is_false() {
    let mut row = base_row();
    row.ssl_enabled = 1;
    row.ssl_verify_server = 2; // not 0 and not 1

    let conn = row.into_connection("pw".into(), None, None);
    let ssl = conn
        .ssl_config
        .expect("ssl_config should be Some when ssl_enabled=1");
    // Production code: `self.ssl_verify_server == 1`, so 2 maps to false
    assert!(!ssl.verify_server);
}
