import { useTranslation } from "react-i18next";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { DB_TYPE_LABELS } from "@/types";
import type { DbType } from "@/types";
import type { FormData } from "./formMapper";

interface BasicTabProps {
  formData: FormData;
  updateField: <K extends keyof FormData>(field: K, value: FormData[K]) => void;
  onDbTypeChange: (dbType: DbType) => void;
}

export function BasicTab({ formData, updateField, onDbTypeChange }: BasicTabProps) {
  const { t } = useTranslation();

  return (
    <>
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
          <Select value={formData.db_type} onValueChange={onDbTypeChange}>
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
    </>
  );
}
