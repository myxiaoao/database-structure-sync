import { useTranslation } from "react-i18next";
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
import type { FormData } from "./formMapper";

interface SshTabProps {
  formData: FormData;
  updateField: <K extends keyof FormData>(field: K, value: FormData[K]) => void;
}

export function SshTab({ formData, updateField }: SshTabProps) {
  const { t } = useTranslation();

  return (
    <>
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
    </>
  );
}
