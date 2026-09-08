export interface Category {
  id: number;
  parent_id: number | null;
  name: string;
  sort: number;
  status: number;
  is_builtin: number;
}

export interface Account {
  id: number;
  name: string;
  sort: number;
  status: number;
}

export type BillType = 1 | 2;

export interface Bill {
  id: number;
  type: BillType;
  amount: number;
  category_id: number;
  account_id: number;
  bill_date: string;
  remark: string;
  source: number;
  status: number;
  created_at: string;
  updated_at: string;
}

export interface BillFilter {
  type?: BillType | null;
  category_id?: number | null;
  account_id?: number | null;
  start_date?: string | null;
  end_date?: string | null;
  keyword?: string | null;
}

export interface BillInput {
  type: BillType;
  amount: number;
  category_id: number;
  account_id: number;
  bill_date: string;
  remark: string;
}

export interface Bootstrap {
  categories: Category[];
  accounts: Account[];
  cycle_rules: CycleRule[];
}

import type { CycleRule } from "./cycle/engine";
export type { CycleInstance, CycleRule, CycleRuleType, LengthUnit } from "./cycle/engine";

export type CycleRuleInput = Omit<CycleRule, "id" | "status"> & { status: number };

export interface Media {
  id: number;
  bill_id: number;
  type: 1 | 2;
  file_name: string;
  ext: string;
  size: number;
  path: string;
  thumb_path: string | null;
}

export interface MediaUsage {
  count: number;
  bytes: number;
}

export interface ImportFile {
  name: string;
  ext: string;
  size: number;
  text: string | null;
  base64: string | null;
  md5: string;
}

export interface ImportItemInput {
  type: 1 | 2;
  amount: number;
  category_id: number;
  bill_date: string;
  remark: string;
}

export interface ConfirmImportInput {
  file_name: string;
  file_md5: string;
  items: ImportItemInput[];
}

export interface ImportTask {
  id: number;
  file_name: string;
  file_md5: string;
  total: number;
  created_at: string;
}

export interface BackupInfo {
  name: string;
  path: string;
  size: number;
  with_media: boolean;
  created_at: string;
}

export interface OrphanInfo {
  count: number;
  bytes: number;
}

export interface ExportResult {
  count: number;
  path: string;
}

export interface AppInfo {
  version: string;
  name: string;
  os: string;
}

export interface AiConfig {
  enabled: boolean;
  hasKey: boolean;
  model: string;
}

export interface AiParsedLine {
  line: number;
  date: string;
  amount: number;
  type: 1 | 2;
  category: string;
  remark: string;
}

export interface CycleRange {
  start: string;
  end: string;
}
