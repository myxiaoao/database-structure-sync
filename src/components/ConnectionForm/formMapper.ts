import type { Connection, ConnectionInput, DbType } from "@/types";

export type SshAuthMethod = "Password" | "PrivateKey";

export interface FormData {
  id?: string;
  name: string;
  db_type: DbType;
  host: string;
  port: number;
  username: string;
  password: string;
  database: string;
  ssh_enabled: boolean;
  ssh_host: string;
  ssh_port: number;
  ssh_username: string;
  ssh_auth_method: SshAuthMethod;
  ssh_password: string;
  ssh_private_key_path: string;
  ssh_passphrase: string;
  ssl_enabled: boolean;
  ssl_ca_cert_path: string;
  ssl_client_cert_path: string;
  ssl_client_key_path: string;
  ssl_verify_server_cert: boolean;
}

export const DEFAULT_FORM_DATA: FormData = {
  name: "",
  db_type: "mysql",
  host: "localhost",
  port: 3306,
  username: "",
  password: "",
  database: "",
  ssh_enabled: false,
  ssh_host: "",
  ssh_port: 22,
  ssh_username: "",
  ssh_auth_method: "Password",
  ssh_password: "",
  ssh_private_key_path: "",
  ssh_passphrase: "",
  ssl_enabled: false,
  ssl_ca_cert_path: "",
  ssl_client_cert_path: "",
  ssl_client_key_path: "",
  ssl_verify_server_cert: true,
};

export const DEFAULT_PORTS: Record<string, number> = {
  mysql: 3306,
  postgresql: 5432,
  mariadb: 3306,
};

export function toConnectionInput(form: FormData): ConnectionInput {
  const input: ConnectionInput = {
    id: form.id,
    name: form.name,
    db_type: form.db_type,
    host: form.host,
    port: form.port,
    username: form.username,
    password: form.password,
    database: form.database,
  };

  if (form.ssh_enabled) {
    input.ssh_config = {
      enabled: true,
      host: form.ssh_host,
      port: form.ssh_port,
      username: form.ssh_username,
      auth_method:
        form.ssh_auth_method === "PrivateKey"
          ? {
              privatekey: {
                private_key_path: form.ssh_private_key_path,
                passphrase: form.ssh_passphrase || undefined,
              },
            }
          : { password: { password: form.ssh_password } },
    };
  }

  if (form.ssl_enabled) {
    input.ssl_config = {
      enabled: true,
      ca_cert_path: form.ssl_ca_cert_path || undefined,
      client_cert_path: form.ssl_client_cert_path || undefined,
      client_key_path: form.ssl_client_key_path || undefined,
      verify_server: form.ssl_verify_server_cert,
    };
  }

  return input;
}

export function fromConnection(conn: Connection): FormData {
  const sshConfig = conn.ssh_config;
  const sslConfig = conn.ssl_config;

  let sshAuthMethod: SshAuthMethod = "Password";
  let sshPassword = "";
  let sshPrivateKeyPath = "";
  let sshPassphrase = "";

  if (sshConfig) {
    if ("privatekey" in sshConfig.auth_method) {
      sshAuthMethod = "PrivateKey";
      sshPrivateKeyPath = sshConfig.auth_method.privatekey.private_key_path;
      sshPassphrase = sshConfig.auth_method.privatekey.passphrase ?? "";
    } else if ("password" in sshConfig.auth_method) {
      sshAuthMethod = "Password";
      sshPassword = sshConfig.auth_method.password.password;
    }
  }

  return {
    id: conn.id,
    name: conn.name,
    db_type: conn.db_type,
    host: conn.host,
    port: conn.port,
    username: conn.username,
    password: conn.password,
    database: conn.database,
    ssh_enabled: sshConfig?.enabled ?? false,
    ssh_host: sshConfig?.host ?? "",
    ssh_port: sshConfig?.port ?? 22,
    ssh_username: sshConfig?.username ?? "",
    ssh_auth_method: sshAuthMethod,
    ssh_password: sshPassword,
    ssh_private_key_path: sshPrivateKeyPath,
    ssh_passphrase: sshPassphrase,
    ssl_enabled: sslConfig?.enabled ?? false,
    ssl_ca_cert_path: sslConfig?.ca_cert_path ?? "",
    ssl_client_cert_path: sslConfig?.client_cert_path ?? "",
    ssl_client_key_path: sslConfig?.client_key_path ?? "",
    ssl_verify_server_cert: sslConfig?.verify_server ?? true,
  };
}
