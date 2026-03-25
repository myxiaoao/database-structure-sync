import { useTranslation } from "react-i18next";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Connection, ConnectionInput } from "@/lib/api";
import { useConnectionForm } from "./useConnectionForm";
import { BasicTab } from "./BasicTab";
import { SshTab } from "./SshTab";
import { SslTab } from "./SslTab";

interface ConnectionFormProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  connection?: Connection | null;
  onSave: (input: ConnectionInput) => Promise<void>;
  onTest: (input: ConnectionInput) => Promise<void>;
}

export function ConnectionForm({
  open,
  onOpenChange,
  connection,
  onSave,
  onTest,
}: ConnectionFormProps) {
  const { t } = useTranslation();
  const {
    formData,
    loading,
    testing,
    testResult,
    isValid,
    updateField,
    handleDbTypeChange,
    handleSave,
    handleTest,
  } = useConnectionForm({ connection, open, onSave, onTest, onOpenChange });

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
          <DialogDescription className="sr-only">
            {connection ? t("connection.edit") : t("connection.new")}
          </DialogDescription>
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
            <BasicTab
              formData={formData}
              updateField={updateField}
              onDbTypeChange={handleDbTypeChange}
            />
          </TabsContent>

          <TabsContent value="ssh" className="space-y-3 mt-3">
            <SshTab formData={formData} updateField={updateField} />
          </TabsContent>

          <TabsContent value="ssl" className="space-y-3 mt-3">
            <SslTab formData={formData} updateField={updateField} />
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
            onClick={() => handleTest(t("connection.testSuccess"))}
            disabled={testing || loading}
            className="h-8"
          >
            {testing ? t("connection.testing") : t("connection.test")}
          </Button>
          <Button
            size="sm"
            onClick={handleSave}
            disabled={!isValid || loading || testing}
            className="h-8"
          >
            {loading ? t("connection.saving") : t("connection.save")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
