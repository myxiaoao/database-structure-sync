import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Connection, ConnectionInput } from "@/lib/api";
import { DB_TYPE_LABELS } from "@/types";
import type { DbType } from "@/types";

type SshAuthMethod = "Password" | "PrivateKey";

interface FormData {
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

const DEFAULT_FORM_DATA: FormData = {
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

function toConnectionInput(form: FormData): ConnectionInput {
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

function fromConnection(conn: Connection): FormData {
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

interface ConnectionFormProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  connection?: Connection | null;
  onSave: (input: ConnectionInput) => Promise<void>;
  onTest: (input: ConnectionInput) => Promise<void>;
}

const DEFAULT_PORTS: Record<string, number> = {
  mysql: 3306,
  postgresql: 5432,
  mariadb: 3306,
};

export function ConnectionForm({
  open,
  onOpenChange,
  connection,
  onSave,
  onTest,
}: ConnectionFormProps) {
  const { t } = useTranslation();
  const [loading, setLoading] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<{ success: boolean; message: string } | null>(null);

  const [formData, setFormData] = useState<FormData>({ ...DEFAULT_FORM_DATA });

  useEffect(() => {
    if (connection) {
      setFormData(fromConnection(connection));
    } else {
      setFormData({ ...DEFAULT_FORM_DATA });
    }
    setTestResult(null);
  }, [connection, open]);

  const updateField = <K extends keyof FormData>(key: K, value: FormData[K]) => {
    setFormData((prev) => ({ ...prev, [key]: value }));
    setTestResult(null);
  };

  const handleDbTypeChange = (value: DbType) => {
    setFormData((prev) => ({
      ...prev,
      db_type: value,
      port: DEFAULT_PORTS[value],
    }));
    setTestResult(null);
  };

  const handleTest = async () => {
    setTesting(true);
    setTestResult(null);
    try {
      await onTest(toConnectionInput(formData));
      setTestResult({ success: true, message: t("connection.testSuccess") });
    } catch (err) {
      setTestResult({
        success: false,
        message: err instanceof Error ? err.message : String(err),
      });
    } finally {
      setTesting(false);
    }
  };

  const canSave =
    formData.name.trim() !== "" &&
    formData.host.trim() !== "" &&
    formData.port > 0 &&
    formData.username.trim() !== "";

  const handleSave = async () => {
    if (!canSave) return;
    setLoading(true);
    try {
      await onSave(toConnectionInput(formData));
      onOpenChange(false);
    } catch (err) {
      setTestResult({
        success: false,
        message: err instanceof Error ? err.message : String(err),
      });
    } finally {
      setLoading(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        className="max-w-xl max-h-[85vh] overflow-y-auto"
        onKeyDown={(e) => {
          if (e.key === "Enter" && e.target instanceof HTMLInputElement) {
            e.preventDefault();
          }
        }}
      >
        <DialogHeader className="pb-2">
          <DialogTitle className="text-base">
            {connection ? t("connection.edit") : t("connection.new")}
          </DialogTitle>
        </DialogHeader>

        <Tabs defaultValue="basic" className="w-full">
          <TabsList className="grid w-full grid-cols-3 h-8">
            <TabsTrigger value="basic" className="text-xs">
              {t("connection.basicTab")}
            </TabsTrigger>
            <TabsTrigger value="ssh" className="text-xs">
              {t("connection.sshTab")}
            </TabsTrigger>
            <TabsTrigger value="ssl" className="text-xs">
              {t("connection.sslTab")}
            </TabsTrigger>
          </TabsList>

          <TabsContent value="basic" className="space-y-3 mt-3">
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-1">
                <Label htmlFor="name" className="text-xs">
                  {t("connection.name")}
                </Label>
                <Input
                  id="name"
                  value={formData.name}
                  onChange={(e) => updateField("name", e.target.value)}
                  placeholder={t("connection.namePlaceholder")}
                  className="h-8 text-sm"
                />
              </div>
              <div className="space-y-1">
                <Label htmlFor="db_type" className="text-xs">
                  {t("connection.dbType")}
                </Label>
                <Select value={formData.db_type} onValueChange={handleDbTypeChange}>
                  <SelectTrigger size="sm" className="text-sm w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="mysql">{DB_TYPE_LABELS.mysql}</SelectItem>
                    <SelectItem value="postgresql">{DB_TYPE_LABELS.postgresql}</SelectItem>
                    <SelectItem value="mariadb">{DB_TYPE_LABELS.mariadb}</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>

            <div className="grid grid-cols-3 gap-3">
              <div className="col-span-2 space-y-1">
                <Label htmlFor="host" className="text-xs">
                  {t("connection.host")}
                </Label>
                <Input
                  id="host"
                  value={formData.host}
                  onChange={(e) => updateField("host", e.target.value)}
                  placeholder="localhost"
                  className="h-8 text-sm"
                />
              </div>
              <div className="space-y-1">
                <Label htmlFor="port" className="text-xs">
                  {t("connection.port")}
                </Label>
                <Input
                  id="port"
                  type="number"
                  value={formData.port}
                  onChange={(e) => updateField("port", parseInt(e.target.value) || 0)}
                  className="h-8 text-sm"
                />
              </div>
            </div>

            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-1">
                <Label htmlFor="username" className="text-xs">
                  {t("connection.username")}
                </Label>
                <Input
                  id="username"
                  value={formData.username}
                  onChange={(e) => updateField("username", e.target.value)}
                  className="h-8 text-sm"
                />
              </div>
              <div className="space-y-1">
                <Label htmlFor="password" className="text-xs">
                  {t("connection.password")}
                </Label>
                <Input
                  id="password"
                  type="password"
                  value={formData.password}
                  onChange={(e) => updateField("password", e.target.value)}
                  className="h-8 text-sm"
                />
              </div>
            </div>

            <div className="space-y-1">
              <Label htmlFor="database" className="text-xs">
                {t("connection.database")}
              </Label>
              <Input
                id="database"
                value={formData.database}
                onChange={(e) => updateField("database", e.target.value)}
                className="h-8 text-sm"
              />
            </div>
          </TabsContent>

          <TabsContent value="ssh" className="space-y-3 mt-3">
            <div className="flex items-center space-x-2">
              <Checkbox
                id="ssh_enabled"
                checked={formData.ssh_enabled}
                onCheckedChange={(checked) => updateField("ssh_enabled", checked === true)}
                className="h-3.5 w-3.5"
              />
              <Label htmlFor="ssh_enabled" className="text-xs">
                {t("connection.sshEnabled")}
              </Label>
            </div>

            {formData.ssh_enabled && (
              <>
                <div className="grid grid-cols-3 gap-3">
                  <div className="col-span-2 space-y-1">
                    <Label htmlFor="ssh_host" className="text-xs">
                      {t("connection.sshHost")}
                    </Label>
                    <Input
                      id="ssh_host"
                      value={formData.ssh_host}
                      onChange={(e) => updateField("ssh_host", e.target.value)}
                      className="h-8 text-sm"
                    />
                  </div>
                  <div className="space-y-1">
                    <Label htmlFor="ssh_port" className="text-xs">
                      {t("connection.sshPort")}
                    </Label>
                    <Input
                      id="ssh_port"
                      type="number"
                      value={formData.ssh_port}
                      onChange={(e) => updateField("ssh_port", parseInt(e.target.value) || 22)}
                      className="h-8 text-sm"
                    />
                  </div>
                </div>

                <div className="space-y-1">
                  <Label htmlFor="ssh_username" className="text-xs">
                    {t("connection.sshUsername")}
                  </Label>
                  <Input
                    id="ssh_username"
                    value={formData.ssh_username}
                    onChange={(e) => updateField("ssh_username", e.target.value)}
                    className="h-8 text-sm"
                  />
                </div>

                <div className="space-y-1">
                  <Label className="text-xs">{t("connection.sshAuthMethod")}</Label>
                  <Select
                    value={formData.ssh_auth_method}
                    onValueChange={(value: "Password" | "PrivateKey") =>
                      updateField("ssh_auth_method", value)
                    }
                  >
                    <SelectTrigger size="sm" className="text-sm w-full">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="Password">{t("connection.sshPassword")}</SelectItem>
                      <SelectItem value="PrivateKey">{t("connection.sshPrivateKey")}</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                {formData.ssh_auth_method === "PrivateKey" ? (
                  <>
                    <div className="space-y-1">
                      <Label htmlFor="ssh_private_key_path" className="text-xs">
                        {t("connection.sshPrivateKeyPath")}
                      </Label>
                      <Input
                        id="ssh_private_key_path"
                        value={formData.ssh_private_key_path}
                        onChange={(e) => updateField("ssh_private_key_path", e.target.value)}
                        placeholder="~/.ssh/id_rsa"
                        className="h-8 text-sm"
                      />
                    </div>
                    <div className="space-y-1">
                      <Label htmlFor="ssh_passphrase" className="text-xs">
                        {t("connection.sshPassphrase")}
                      </Label>
                      <Input
                        id="ssh_passphrase"
                        type="password"
                        value={formData.ssh_passphrase}
                        onChange={(e) => updateField("ssh_passphrase", e.target.value)}
                        className="h-8 text-sm"
                      />
                    </div>
                  </>
                ) : (
                  <div className="space-y-1">
                    <Label htmlFor="ssh_password" className="text-xs">
                      {t("connection.sshPasswordField")}
                    </Label>
                    <Input
                      id="ssh_password"
                      type="password"
                      value={formData.ssh_password}
                      onChange={(e) => updateField("ssh_password", e.target.value)}
                      className="h-8 text-sm"
                    />
                  </div>
                )}
              </>
            )}
          </TabsContent>

          <TabsContent value="ssl" className="space-y-3 mt-3">
            <div className="flex items-center space-x-2">
              <Checkbox
                id="ssl_enabled"
                checked={formData.ssl_enabled}
                onCheckedChange={(checked) => updateField("ssl_enabled", checked === true)}
                className="h-3.5 w-3.5"
              />
              <Label htmlFor="ssl_enabled" className="text-xs">
                {t("connection.sslEnabled")}
              </Label>
            </div>

            {formData.ssl_enabled && (
              <>
                <div className="space-y-1">
                  <Label htmlFor="ssl_ca_cert_path" className="text-xs">
                    {t("connection.sslCaCert")}
                  </Label>
                  <Input
                    id="ssl_ca_cert_path"
                    value={formData.ssl_ca_cert_path}
                    onChange={(e) => updateField("ssl_ca_cert_path", e.target.value)}
                    placeholder="/path/to/ca-cert.pem"
                    className="h-8 text-sm"
                  />
                </div>
                <div className="space-y-1">
                  <Label htmlFor="ssl_client_cert_path" className="text-xs">
                    {t("connection.sslClientCert")}
                  </Label>
                  <Input
                    id="ssl_client_cert_path"
                    value={formData.ssl_client_cert_path}
                    onChange={(e) => updateField("ssl_client_cert_path", e.target.value)}
                    placeholder="/path/to/client-cert.pem"
                    className="h-8 text-sm"
                  />
                </div>
                <div className="space-y-1">
                  <Label htmlFor="ssl_client_key_path" className="text-xs">
                    {t("connection.sslClientKey")}
                  </Label>
                  <Input
                    id="ssl_client_key_path"
                    value={formData.ssl_client_key_path}
                    onChange={(e) => updateField("ssl_client_key_path", e.target.value)}
                    placeholder="/path/to/client-key.pem"
                    className="h-8 text-sm"
                  />
                </div>
                <div className="flex items-center space-x-2">
                  <Checkbox
                    id="ssl_verify_server_cert"
                    checked={formData.ssl_verify_server_cert}
                    onCheckedChange={(checked) =>
                      updateField("ssl_verify_server_cert", checked === true)
                    }
                    className="h-3.5 w-3.5"
                  />
                  <Label htmlFor="ssl_verify_server_cert" className="text-xs">
                    {t("connection.sslVerifyServerCert")}
                  </Label>
                </div>
              </>
            )}
          </TabsContent>
        </Tabs>

        {testResult && (
          <div
            className={`p-2 rounded text-xs ${
              testResult.success
                ? "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200"
                : "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200"
            }`}
          >
            {testResult.message}
          </div>
        )}

        <DialogFooter className="gap-2 pt-2">
          <Button
            variant="outline"
            size="sm"
            onClick={handleTest}
            disabled={testing || loading}
            className="h-8"
          >
            {testing ? t("connection.testing") : t("connection.test")}
          </Button>
          <Button
            size="sm"
            onClick={handleSave}
            disabled={!canSave || loading || testing}
            className="h-8"
          >
            {loading ? t("connection.saving") : t("connection.save")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
