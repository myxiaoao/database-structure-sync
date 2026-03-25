import { useState, useEffect } from "react";
import {
  type FormData,
  DEFAULT_FORM_DATA,
  DEFAULT_PORTS,
  fromConnection,
  toConnectionInput,
} from "./formMapper";
import type { Connection, ConnectionInput, DbType } from "@/types";

interface UseConnectionFormOptions {
  connection?: Connection | null;
  open: boolean;
  onSave: (input: ConnectionInput) => Promise<void>;
  onTest: (input: ConnectionInput) => Promise<void>;
  onOpenChange: (open: boolean) => void;
}

export function useConnectionForm({
  connection,
  open,
  onSave,
  onTest,
  onOpenChange,
}: UseConnectionFormOptions) {
  const [formData, setFormData] = useState<FormData>({ ...DEFAULT_FORM_DATA });
  const [loading, setLoading] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<{
    success: boolean;
    message: string;
  } | null>(null);

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

  const isValid =
    formData.name.trim() !== "" &&
    formData.host.trim() !== "" &&
    formData.port > 0 &&
    formData.username.trim() !== "";

  const handleSave = async () => {
    if (!isValid) return;
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

  const handleTest = async (successMessage: string) => {
    setTesting(true);
    setTestResult(null);
    try {
      await onTest(toConnectionInput(formData));
      setTestResult({ success: true, message: successMessage });
    } catch (err) {
      setTestResult({
        success: false,
        message: err instanceof Error ? err.message : String(err),
      });
    } finally {
      setTesting(false);
    }
  };

  return {
    formData,
    loading,
    testing,
    testResult,
    isValid,
    updateField,
    handleDbTypeChange,
    handleSave,
    handleTest,
  };
}
