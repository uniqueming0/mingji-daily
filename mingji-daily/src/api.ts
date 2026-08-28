import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type {
  Account,
  BackupInfo,
  Bill,
  BillFilter,
  BillInput,
  Bootstrap,
  Category,
  ConfirmImportInput,
  CycleRule,
  CycleRuleInput,
  ExportResult,
  ImportFile,
  ImportTask,
  Media,
  MediaUsage,
  OrphanInfo,
} from "./types";

export const api = {
  bootstrap: () => invoke<Bootstrap>("get_bootstrap"),
  getDataDir: () => invoke<string>("get_data_dir"),

  listBills: (filter?: BillFilter) => invoke<Bill[]>("list_bills", { filter: filter ?? null }),
  createBill: (input: BillInput) => invoke<Bill>("create_bill", { input }),
  updateBill: (id: number, input: BillInput) => invoke<Bill>("update_bill", { id, input }),
  deleteBill: (id: number) => invoke<void>("delete_bill", { id }),

  createCategory: (name: string, parentId: number | null) =>
    invoke<Category>("create_category", { name, parentId }),
  updateCategory: (id: number, name: string, status: number) =>
    invoke<Category>("update_category", { id, name, status }),
  deleteCategory: (id: number) => invoke<void>("delete_category", { id }),

  createAccount: (name: string) => invoke<Account>("create_account", { name }),
  updateAccount: (id: number, name: string, status: number) =>
    invoke<Account>("update_account", { id, name, status }),
  deleteAccount: (id: number) => invoke<void>("delete_account", { id }),

  createCycleRule: (input: CycleRuleInput) => invoke<CycleRule>("create_cycle_rule", { input }),
  updateCycleRule: (id: number, input: CycleRuleInput) =>
    invoke<CycleRule>("update_cycle_rule", { id, input }),
  deleteCycleRule: (id: number) => invoke<void>("delete_cycle_rule", { id }),

  attachMedia: (billId: number, path: string) => invoke<Media>("attach_media", { billId, path }),
  listMedia: (billId: number) => invoke<Media[]>("list_media", { billId }),
  deleteMedia: (id: number) => invoke<void>("delete_media", { id }),
  getMediaUsage: () => invoke<MediaUsage>("get_media_usage"),

  readImportFile: (path: string) => invoke<ImportFile>("read_import_file", { path }),
  checkImportDuplicate: (fileMd5: string) =>
    invoke<ImportTask | null>("check_import_duplicate", { fileMd5 }),
  confirmImport: (input: ConfirmImportInput) => invoke<ImportTask>("confirm_import", { input }),
  listImportTasks: () => invoke<ImportTask[]>("list_import_tasks"),
  revokeImport: (taskId: number) => invoke<void>("revoke_import", { taskId }),

  createBackup: (withMedia: boolean) => invoke<BackupInfo>("create_backup", { withMedia }),
  listBackups: () => invoke<BackupInfo[]>("list_backups"),
  restoreBackup: (name: string) => invoke<void>("restore_backup", { name }),
  deleteBackup: (name: string) => invoke<void>("delete_backup", { name }),
  exportBillsCsv: (path: string) => invoke<ExportResult>("export_bills_csv", { path }),
  exportAllJson: (path: string) => invoke<ExportResult>("export_all_json", { path }),
  getOrphanMedia: () => invoke<OrphanInfo>("get_orphan_media"),
  cleanOrphanMedia: () => invoke<OrphanInfo>("clean_orphan_media"),
  checkDbIntegrity: () => invoke<string>("check_db_integrity"),
  migrateDataDir: (target: string) => invoke<string>("migrate_data_dir", { target }),
};

/** 本地文件路径 → 可被 <img>/<video> 使用的 asset 协议 URL */
export function mediaSrc(path: string): string {
  return convertFileSrc(path);
}
