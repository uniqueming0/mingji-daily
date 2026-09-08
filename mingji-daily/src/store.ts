import { reactive } from "vue";
import { api } from "./api";
import type { Account, AiConfig, Category, CycleRule } from "./types";

export const store = reactive({
  ready: false,
  categories: [] as Category[],
  accounts: [] as Account[],
  cycleRules: [] as CycleRule[],
});

export const aiConfig = reactive<AiConfig>({ enabled: false, hasKey: false, model: "deepseek-chat" });

export async function refreshAiConfig() {
  try {
    const cfg = await api.getAiConfig();
    aiConfig.enabled = cfg.enabled;
    aiConfig.hasKey = cfg.hasKey;
    aiConfig.model = cfg.model;
  } catch {
    /* 忽略 */
  }
}

export async function bootstrap() {
  const data = await api.bootstrap();
  store.categories = data.categories;
  store.accounts = data.accounts;
  store.cycleRules = data.cycle_rules;
  store.ready = true;
}

export async function reloadMeta() {
  const data = await api.bootstrap();
  store.categories = data.categories;
  store.accounts = data.accounts;
  store.cycleRules = data.cycle_rules;
}

// ---------- 跨页面导航（统计页 → 明细页穿透） ----------

export interface BillsNavFilter {
  category_id?: number | null;
  account_id?: number | null;
  type?: 1 | 2 | null;
  start_date?: string | null;
  end_date?: string | null;
  keyword?: string | null;
}

export const navRequest = reactive({
  seq: 0,
  filter: null as BillsNavFilter | null,
});

export function gotoBills(filter: BillsNavFilter) {
  navRequest.filter = filter;
  navRequest.seq++;
}

export function categoryName(id: number): string {
  return store.categories.find((c) => c.id === id)?.name ?? "未分类";
}

export function accountName(id: number): string {
  return store.accounts.find((a) => a.id === id)?.name ?? "未知账户";
}

export function topCategories(): Category[] {
  return store.categories.filter((c) => c.parent_id === null && c.status === 1);
}

export function childrenOf(parentId: number): Category[] {
  return store.categories.filter((c) => c.parent_id === parentId && c.status === 1);
}
