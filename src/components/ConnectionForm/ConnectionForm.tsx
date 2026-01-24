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

interface ConnectionFormProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  connection?: Connection | null;
  onSave: (input: ConnectionInput) => Promise<void>;
  onTest: (input: ConnectionInput) => Promise<void>;
}

const DEFAULT_PORTS: Record<string, number> = {
  MySQL: 3306,
  PostgreSQL: 5432,
  MariaDB: 3306,
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

  const [formData, setFormData] = useState<ConnectionInput>({
    name: "",
    db_type: "MySQL",
    host: "localhost",
    port: 3306,
    username: "",
    password: "",
    database: "",
    ssh_enabled: false,
    ssl_enabled: false,
  });

  useEffect(() => {
    if (connection) {
      setFormData({
        id: connection.id,
        name: connection.name,
        db_type: connection.db_type,
        host: connection.host,
        port: connection.port,
        username: connection.username,
        password: connection.password,
        database: connection.database,
        ssh_enabled: connection.ssh_enabled,
        ssh_host: connection.ssh_host,
        ssh_port: connection.ssh_port,
        ssh_username: connection.ssh_username,
        ssh_auth_method: connection.ssh_auth_method,
        ssh_password: connection.ssh_password,
        ssh_private_key_path: connection.ssh_private_key_path,
        ssh_passphrase: connection.ssh_passphrase,
        ssl_enabled: connection.ssl_enabled,
        ssl_ca_cert_path: connection.ssl_ca_cert_path,
        ssl_client_cert_path: connection.ssl_client_cert_path,
        ssl_client_key_path: connection.ssl_client_key_path,
        ssl_verify_server_cert: connection.ssl_verify_server_cert,
      });
    } else {
      setFormData({
        name: "",
        db_type: "MySQL",
        host: "localhost",
        port: 3306,
        username: "",
        password: "",
        database: "",
        ssh_enabled: false,
        ssl_enabled: false,
      });
    }
    setTestResult(null);
  }, [connection, open]);

  const updateField = <K extends keyof ConnectionInput>(key: K, value: ConnectionInput[K]) => {
    setFormData((prev) => ({ ...prev, [key]: value }));
    setTestResult(null);
  };

  const handleDbTypeChange = (value: "MySQL" | "PostgreSQL" | "MariaDB") => {
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
      await onTest(formData);
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

  const handleSave = async () => {
    setLoading(true);
    try {
      await onSave(formData);
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
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{connection ? t("connection.edit") : t("connection.new")}</DialogTitle>
        </DialogHeader>

        <Tabs defaultValue="basic" className="w-full">
          <TabsList className="grid w-full grid-cols-3">
            <TabsTrigger value="basic">{t("connection.basicTab")}</TabsTrigger>
            <TabsTrigger value="ssh">{t("connection.sshTab")}</TabsTrigger>
            <TabsTrigger value="ssl">{t("connection.sslTab")}</TabsTrigger>
          </TabsList>

          <TabsContent value="basic" className="space-y-4 mt-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="name">{t("connection.name")}</Label>
                <Input
                  id="name"
                  value={formData.name}
                  onChange={(e) => updateField("name", e.target.value)}
                  placeholder={t("connection.namePlaceholder")}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="db_type">{t("connection.dbType")}</Label>
                <Select value={formData.db_type} onValueChange={handleDbTypeChange}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="MySQL">MySQL</SelectItem>
                    <SelectItem value="PostgreSQL">PostgreSQL</SelectItem>
                    <SelectItem value="MariaDB">MariaDB</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>

            <div className="grid grid-cols-3 gap-4">
              <div className="col-span-2 space-y-2">
                <Label htmlFor="host">{t("connection.host")}</Label>
                <Input
                  id="host"
                  value={formData.host}
                  onChange={(e) => updateField("host", e.target.value)}
                  placeholder="localhost"
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="port">{t("connection.port")}</Label>
                <Input
                  id="port"
                  type="number"
                  value={formData.port}
                  onChange={(e) => updateField("port", parseInt(e.target.value) || 0)}
                />
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="username">{t("connection.username")}</Label>
                <Input
                  id="username"
                  value={formData.username}
                  onChange={(e) => updateField("username", e.target.value)}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="password">{t("connection.password")}</Label>
                <Input
                  id="password"
                  type="password"
                  value={formData.password}
                  onChange={(e) => updateField("password", e.target.value)}
                />
              </div>
            </div>

            <div className="space-y-2">
              <Label htmlFor="database">{t("connection.database")}</Label>
              <Input
                id="database"
                value={formData.database}
                onChange={(e) => updateField("database", e.target.value)}
              />
            </div>
          </TabsContent>

          <TabsContent value="ssh" className="space-y-4 mt-4">
            <div className="flex items-center space-x-2">
              <Checkbox
                id="ssh_enabled"
                checked={formData.ssh_enabled}
                onCheckedChange={(checked) => updateField("ssh_enabled", checked === true)}
              />
              <Label htmlFor="ssh_enabled">{t("connection.sshEnabled")}</Label>
            </div>

            {formData.ssh_enabled && (
              <>
                <div className="grid grid-cols-3 gap-4">
                  <div className="col-span-2 space-y-2">
                    <Label htmlFor="ssh_host">{t("connection.sshHost")}</Label>
                    <Input
                      id="ssh_host"
                      value={formData.ssh_host || ""}
                      onChange={(e) => updateField("ssh_host", e.target.value)}
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="ssh_port">{t("connection.sshPort")}</Label>
                    <Input
                      id="ssh_port"
                      type="number"
                      value={formData.ssh_port || 22}
                      onChange={(e) => updateField("ssh_port", parseInt(e.target.value) || 22)}
                    />
                  </div>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="ssh_username">{t("connection.sshUsername")}</Label>
                  <Input
                    id="ssh_username"
                    value={formData.ssh_username || ""}
                    onChange={(e) => updateField("ssh_username", e.target.value)}
                  />
                </div>

                <div className="space-y-2">
                  <Label>{t("connection.sshAuthMethod")}</Label>
                  <Select
                    value={formData.ssh_auth_method || "Password"}
                    onValueChange={(value: "Password" | "PrivateKey") =>
                      updateField("ssh_auth_method", value)
                    }
                  >
                    <SelectTrigger>
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
                    <div className="space-y-2">
                      <Label htmlFor="ssh_private_key_path">
                        {t("connection.sshPrivateKeyPath")}
                      </Label>
                      <Input
                        id="ssh_private_key_path"
                        value={formData.ssh_private_key_path || ""}
                        onChange={(e) => updateField("ssh_private_key_path", e.target.value)}
                        placeholder="~/.ssh/id_rsa"
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="ssh_passphrase">{t("connection.sshPassphrase")}</Label>
                      <Input
                        id="ssh_passphrase"
                        type="password"
                        value={formData.ssh_passphrase || ""}
                        onChange={(e) => updateField("ssh_passphrase", e.target.value)}
                      />
                    </div>
                  </>
                ) : (
                  <div className="space-y-2">
                    <Label htmlFor="ssh_password">{t("connection.sshPasswordField")}</Label>
                    <Input
                      id="ssh_password"
                      type="password"
                      value={formData.ssh_password || ""}
                      onChange={(e) => updateField("ssh_password", e.target.value)}
                    />
                  </div>
                )}
              </>
            )}
          </TabsContent>

          <TabsContent value="ssl" className="space-y-4 mt-4">
            <div className="flex items-center space-x-2">
              <Checkbox
                id="ssl_enabled"
                checked={formData.ssl_enabled}
                onCheckedChange={(checked) => updateField("ssl_enabled", checked === true)}
              />
              <Label htmlFor="ssl_enabled">{t("connection.sslEnabled")}</Label>
            </div>

            {formData.ssl_enabled && (
              <>
                <div className="space-y-2">
                  <Label htmlFor="ssl_ca_cert_path">{t("connection.sslCaCert")}</Label>
                  <Input
                    id="ssl_ca_cert_path"
                    value={formData.ssl_ca_cert_path || ""}
                    onChange={(e) => updateField("ssl_ca_cert_path", e.target.value)}
                    placeholder="/path/to/ca-cert.pem"
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="ssl_client_cert_path">{t("connection.sslClientCert")}</Label>
                  <Input
                    id="ssl_client_cert_path"
                    value={formData.ssl_client_cert_path || ""}
                    onChange={(e) => updateField("ssl_client_cert_path", e.target.value)}
                    placeholder="/path/to/client-cert.pem"
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="ssl_client_key_path">{t("connection.sslClientKey")}</Label>
                  <Input
                    id="ssl_client_key_path"
                    value={formData.ssl_client_key_path || ""}
                    onChange={(e) => updateField("ssl_client_key_path", e.target.value)}
                    placeholder="/path/to/client-key.pem"
                  />
                </div>
                <div className="flex items-center space-x-2">
                  <Checkbox
                    id="ssl_verify_server_cert"
                    checked={formData.ssl_verify_server_cert ?? true}
                    onCheckedChange={(checked) =>
                      updateField("ssl_verify_server_cert", checked === true)
                    }
                  />
                  <Label htmlFor="ssl_verify_server_cert">
                    {t("connection.sslVerifyServerCert")}
                  </Label>
                </div>
              </>
            )}
          </TabsContent>
        </Tabs>

        {testResult && (
          <div
            className={`p-3 rounded-md text-sm ${
              testResult.success
                ? "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200"
                : "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200"
            }`}
          >
            {testResult.message}
          </div>
        )}

        <DialogFooter className="gap-2">
          <Button variant="outline" onClick={handleTest} disabled={testing || loading}>
            {testing ? t("connection.testing") : t("connection.test")}
          </Button>
          <Button onClick={handleSave} disabled={loading || testing}>
            {loading ? t("connection.saving") : t("connection.save")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
