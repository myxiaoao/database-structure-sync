import { useTranslation } from "react-i18next";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import type { FormData } from "./formMapper";

interface SslTabProps {
  formData: FormData;
  updateField: <K extends keyof FormData>(field: K, value: FormData[K]) => void;
}

export function SslTab({ formData, updateField }: SslTabProps) {
  const { t } = useTranslation();

  return (
    <>
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
              onCheckedChange={(checked) => updateField("ssl_verify_server_cert", checked === true)}
              className="h-3.5 w-3.5"
            />
            <Label htmlFor="ssl_verify_server_cert" className="text-xs">
              {t("connection.sslVerifyServerCert")}
            </Label>
          </div>
        </>
      )}
    </>
  );
}
