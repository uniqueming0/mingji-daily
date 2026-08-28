// =============================================================
// 周期规则引擎（纯函数，桌面版与小程序版共用）
// 口径与《本地桌面版需求文档.md》§7.2 一致：
// - 周期边界含头含尾（起始日 00:00:00 ~ 结束日 23:59:59）
// - 月起始日仅支持 1~28 日，天然规避 29/30/31 日的短月顺延问题
// =============================================================

export type CycleRuleType = "week" | "month" | "custom";
export type LengthUnit = "day" | "week" | "month";

export interface CycleRule {
  id: number;
  name: string;
  type: CycleRuleType;
  start_date: string; // yyyy-mm-dd，规则起始日
  length: number; // 自定义周期长度
  length_unit: LengthUnit; // 自定义周期单位
  week_start: number; // 1=周一 … 7=周日
  month_start_day: number; // 1~28
  end_date: string | null; // null = 长期有效
  status: number; // 1 启用 / 0 停用
}

export interface CycleInstance {
  rule_id: number;
  start: string; // 含
  end: string; // 含
  label: string;
}

export type CycleRuleDraft = Omit<CycleRule, "id" | "status">;

function pad(n: number): string {
  return n < 10 ? "0" + n : "" + n;
}

function parseDate(s: string): Date {
  const [y, m, d] = s.split("-").map(Number);
  return new Date(y, m - 1, d);
}

function formatDate(d: Date): string {
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

function addDays(s: string, n: number): string {
  const d = parseDate(s);
  d.setDate(d.getDate() + n);
  return formatDate(d);
}

/** 加 n 个月，日数超出目标月时钳制到当月最后一天 */
export function addMonthsClamped(s: string, n: number): string {
  const [y, m, day] = s.split("-").map(Number);
  const t = new Date(y, m - 1 + n, 1);
  const last = new Date(t.getFullYear(), t.getMonth() + 1, 0).getDate();
  return `${t.getFullYear()}-${pad(t.getMonth() + 1)}-${pad(Math.min(day, last))}`;
}

/** 1=周一 … 7=周日 */
function dowMon(s: string): number {
  return (parseDate(s).getDay() + 6) % 7 + 1;
}

/** 包含 startDate 的"周"的起始日（按 weekStart 对齐） */
function weekAnchor(startDate: string, weekStart: number): string {
  return addDays(startDate, -((dowMon(startDate) - weekStart + 7) % 7));
}

/** 包含 date 的"月周期"的起始日（按 monthStartDay 对齐） */
function monthInstanceStart(date: string, monthStartDay: number): string {
  const [y, m, d] = date.split("-").map(Number);
  if (d >= monthStartDay) return `${y}-${pad(m)}-${pad(monthStartDay)}`;
  const prev = new Date(y, m - 2, 1);
  return `${prev.getFullYear()}-${pad(prev.getMonth() + 1)}-${pad(monthStartDay)}`;
}

/** 单步前进/后退一个周期 */
function periodStep(rule: CycleRule, s: string, dir: 1 | -1): string {
  if (rule.type === "week") return addDays(s, 7 * dir);
  if (rule.type === "month") return addMonthsClamped(s, dir);
  if (rule.length_unit === "month") return addMonthsClamped(s, rule.length * dir);
  return addDays(s, (rule.length_unit === "week" ? rule.length * 7 : rule.length) * dir);
}

/**
 * 展开与 [rangeStart, rangeEnd] 有交集的周期实例（含头含尾）。
 * 规则起始日只是第一个实例的锚点：实例向过去与未来双向无限延伸，
 * 确保规则创建之前的账单也能落入对应周期。
 */
export function expandInstances(
  rule: CycleRule,
  rangeStart: string,
  rangeEnd: string
): CycleInstance[] {
  if (rule.status === 0 || rangeStart > rangeEnd) return [];
  const out: CycleInstance[] = [];

  // 锚点 = 包含规则起始日的那个实例的起始日
  let anchorStart: string;
  if (rule.type === "week") anchorStart = weekAnchor(rule.start_date, rule.week_start);
  else if (rule.type === "month") anchorStart = monthInstanceStart(rule.start_date, rule.month_start_day);
  else anchorStart = rule.start_date;

  // 向后扩展到 rangeStart（覆盖规则创建前的历史账单）
  let cursor = anchorStart;
  for (let i = 0; i < 2000 && cursor > rangeStart; i++) {
    const prev = periodStep(rule, cursor, -1);
    if (prev >= cursor) break; // 防御：步进无效时终止
    cursor = prev;
  }

  // 从 cursor 向前收集
  let start = cursor;
  for (let i = 0; i < 2000; i++) {
    if (start > rangeEnd) break;
    if (rule.end_date && start > rule.end_date) break;
    const end = addDays(periodStep(rule, start, 1), -1);
    if (end >= rangeStart) {
      out.push({ rule_id: rule.id, start, end, label: `${start} ~ ${end}` });
    }
    start = periodStep(rule, start, 1);
  }
  return out;
}

/** 指定日期所属的周期实例（无则 null） */
export function instanceContaining(rule: CycleRule, date: string): CycleInstance | null {
  return expandInstances(rule, date, date)[0] ?? null;
}

const WEEK_NAMES = ["", "周一", "周二", "周三", "周四", "周五", "周六", "周日"];
const UNIT_NAMES: Record<LengthUnit, string> = { day: "天", week: "周", month: "个月" };

export function describeRule(rule: CycleRule): string {
  if (rule.type === "week") return `按周（${WEEK_NAMES[rule.week_start] || "周一"}起始）`;
  if (rule.type === "month") return `按月（每月 ${rule.month_start_day} 日起）`;
  return `自定义（每 ${rule.length} ${UNIT_NAMES[rule.length_unit]}）`;
}

/** 预览：从"今天（或规则起始日，取较晚者）所在周期"起的连续 count 个周期 */
export function previewInstances(draft: CycleRuleDraft, count = 3): CycleInstance[] {
  const today = formatDate(new Date());
  const anchor = draft.start_date > today ? draft.start_date : today;
  const rule: CycleRule = { ...draft, id: 0, status: 1 };
  return expandInstances(rule, anchor, addMonthsClamped(anchor, 24)).slice(0, count);
}
