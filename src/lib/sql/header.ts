import { DB_TYPE_LABELS } from "@/types";
import type { Connection, DbType } from "@/types";

const APP_VERSION = __APP_VERSION__;

export function generateSqlHeader(
  sourceConn: Connection | undefined,
  targetConn: Connection | undefined,
  sourceDb: string,
  targetDb: string,
  itemCount: number
): string {
  const now = new Date();
  const timestamp = now
    .toISOString()
    .replace("T", " ")
    .replace(/\.\d+Z$/, "");
  const sourceName = sourceConn
    ? `${sourceConn.name} (${sourceConn.host}:${sourceConn.port}/${sourceDb || sourceConn.database})`
    : "N/A";
  const targetName = targetConn
    ? `${targetConn.name} (${targetConn.host}:${targetConn.port}/${targetDb || targetConn.database})`
    : "N/A";
  const dbType = targetConn?.db_type || sourceConn?.db_type || "Unknown";
  const isMySQL = dbType === "mysql" || dbType === "mariadb";

  const lines = [
    "-- ---------------------------------------------------------",
    `-- Database Structure Sync v${APP_VERSION}`,
    "--",
    `-- Generation Time: ${timestamp}`,
    `-- Database Type:   ${DB_TYPE_LABELS[dbType as DbType] || dbType}`,
    `-- Source:          ${sourceName}`,
    `-- Target:          ${targetName}`,
    `-- Changes:         ${itemCount} item(s)`,
    "-- ---------------------------------------------------------",
    "",
  ];

  if (isMySQL) {
    lines.push(
      "/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;",
      "/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;",
      "/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;",
      "/*!40101 SET NAMES utf8mb4 */;",
      "/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;",
      "/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;",
      ""
    );
  } else {
    lines.push(
      "SET statement_timeout = 0;",
      "SET lock_timeout = 0;",
      "SET client_encoding = 'UTF8';",
      ""
    );
  }

  return lines.join("\n");
}

export function generateSqlFooter(dbType: string): string {
  const isMySQL = dbType === "mysql" || dbType === "mariadb";

  const lines = [""];

  if (isMySQL) {
    lines.push(
      "/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;",
      "/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;",
      "/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;",
      "/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;",
      "/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;"
    );
  }

  lines.push(
    "",
    "-- ---------------------------------------------------------",
    "-- End of synchronization script",
    "-- ---------------------------------------------------------",
    ""
  );

  return lines.join("\n");
}
