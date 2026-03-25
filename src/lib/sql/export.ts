import { save } from "@tauri-apps/plugin-dialog";
import { syncApi } from "@/lib/api/sync";

export interface ExportSqlOptions {
  defaultPath?: string;
}

export async function exportSqlFile(
  sqlContent: string,
  options: ExportSqlOptions = {}
): Promise<boolean> {
  if (!sqlContent) return false;

  const { defaultPath = "sync.sql" } = options;

  const filePath = await save({
    defaultPath,
    filters: [{ name: "SQL", extensions: ["sql"] }],
  });

  if (!filePath) {
    return false;
  }

  await syncApi.saveSqlFile(filePath, sqlContent);
  return true;
}
