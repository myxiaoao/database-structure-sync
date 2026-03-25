import { save } from "@tauri-apps/plugin-dialog";
import { syncApi } from "@/lib/api/sync";

export async function exportSqlFile(sqlContent: string): Promise<boolean> {
  if (!sqlContent) return false;

  const filePath = await save({
    defaultPath: "sync.sql",
    filters: [{ name: "SQL", extensions: ["sql"] }],
  });

  if (!filePath) {
    return false;
  }

  await syncApi.saveSqlFile(filePath, sqlContent);
  return true;
}
