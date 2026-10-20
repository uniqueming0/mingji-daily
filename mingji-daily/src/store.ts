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

/** 运行平台：windows / android / ios …（移动端隐藏桌面专属功能） */
export const appEnv = reactive({ platform: "windows", isMobile: false });

export async function loadPlatform() {
  try {
    const p = await api.getPlatform();
    appEnv.platform = p;
    appEnv.isMobile = p === "android" || p === "ios";
  } catch {
    /* 忽略 */
  }
}

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
  /** 来源页：从统计页穿透过来时记录，用于在明细页显示"返回统计" */
  from: null as "stats" | null,
});

export function gotoBills(filter: BillsNavFilter, from: "stats" | null = null) {
  navRequest.filter = filter;
  navRequest.from = from;
  navRequest.seq++;
}

/** 当前激活的页签（放在共享状态里，便于跨页面跳转，如"返回统计"） */
export const ui = reactive({ active: "记一笔" });

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
