import { save } from "@tauri-apps/plugin-dialog";
import { syncApi } from "@/lib/api/sync";

export interface ExportSqlOptions {
  defaultPath?: string;
}

function generateDefaultFileName(): string {
  const now = new Date();
  const ts = [
    now.getFullYear(),
    String(now.getMonth() + 1).padStart(2, "0"),
    String(now.getDate()).padStart(2, "0"),
    String(now.getHours()).padStart(2, "0"),
    String(now.getMinutes()).padStart(2, "0"),
    String(now.getSeconds()).padStart(2, "0"),
  ].join("");
  return `sync_${ts}.sql`;
}

export async function exportSqlFile(
  sqlContent: string,
  options: ExportSqlOptions = {}
): Promise<boolean> {
  if (!sqlContent) return false;

  const { defaultPath = generateDefaultFileName() } = options;

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
