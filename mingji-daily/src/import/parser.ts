// =============================================================
// 账单导入解析引擎（纯函数，桌面版与小程序版共用）
// 规则口径与《本地桌面版需求文档.md》附录 A 一致
// =============================================================

export interface CategoryRef {
  id: number;
  name: string;
}

export interface YearRange {
  startYear: number;
  startMonth: number;
  startDay: number;
  endMonth: number;
}

export interface ParsedItem {
  index: number;
  raw: string;
  ok: boolean;
  warn: string;
  date: string;
  amountFen: number;
  type: 1 | 2;
  categoryId: number | null;
  remark: string;
}

export interface ParseOptions {
  today: string; // yyyy-mm-dd
  categories: CategoryRef[];
  yearRange?: YearRange | null; // 由文件标题推断（如 # 账本2025.09.01-09.15）
}

const RELATIVE_DAYS: Record<string, number> = { 今天: 0, 昨天: -1, 前天: -2 };
const INCOME_WORDS = ["收入", "入账", "工资", "奖金", "兼职", "报销", "退款", "利息", "红包", "收到", "转入"];
const KEYWORD_MAP: Array<[string, string]> = [
  ["地铁", "交通"], ["公交", "交通"], ["打车", "交通"], ["滴滴", "交通"], ["加油", "交通"],
  ["停车", "交通"], ["火车", "交通"], ["机票", "交通"], ["高铁", "交通"], ["单车", "交通"],
  ["外卖", "餐饮"], ["奶茶", "餐饮"], ["咖啡", "餐饮"], ["聚餐", "餐饮"], ["零食", "餐饮"],
  ["早饭", "餐饮"], ["午饭", "餐饮"], ["晚饭", "餐饮"], ["早餐", "餐饮"], ["午餐", "餐饮"], ["晚餐", "餐饮"],
  ["话费", "通讯"], ["网费", "通讯"], ["房租", "居住"], ["水电", "居住"], ["燃气", "居住"],
  ["物业", "居住"], ["水费", "居住"], ["电费", "居住"],
  ["电影", "娱乐"], ["游戏", "娱乐"], ["旅行", "娱乐"], ["健身", "娱乐"], ["演出", "娱乐"], ["麻将", "娱乐"],
  ["药品", "医疗"], ["门诊", "医疗"], ["体检", "医疗"], ["挂号", "医疗"],
  ["书籍", "教育"], ["课程", "教育"], ["考试", "教育"], ["论文", "教育"],
  ["洗衣", "购物"],
];

function pad(n: number | string): string {
  const num = Number(n);
  return num < 10 ? "0" + num : "" + num;
}

function formatDate(d: Date): string {
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

function shiftDate(date: string, offset: number): string {
  const d = new Date(date);
  d.setDate(d.getDate() + offset);
  return formatDate(d);
}

/** 缺失月份时该用哪一年：优先取账本标题年份（跨年账本时，起始月之前的月份属于下一年） */
function yearForMonth(month: number, opts: ParseOptions): number {
  const yr = opts.yearRange;
  if (!yr) return Number(opts.today.slice(0, 4));
  const crossYear = yr.endMonth < yr.startMonth;
  return month < yr.startMonth && crossYear ? yr.startYear + 1 : yr.startYear;
}

/** 整行完全没有日期时的兜底日期与提示 */
function fallbackDate(opts: ParseOptions): { date: string; warn: string } {
  if (opts.yearRange) {
    const yr = opts.yearRange;
    return {
      date: `${yr.startYear}-${pad(yr.startMonth)}-${pad(yr.startDay)}`,
      warn: "未识别到日期，已默认账本起始日",
    };
  }
  return { date: opts.today, warn: "未识别到日期，已默认今天" };
}

/** 从标题提取账本日期范围：支持 # 账本2025.09.01-09.15 与 # 账本2025.09.01-2025.09.15 */
function extractYearRange(title: string): YearRange | null {
  let m = title.match(
    /(\d{4})[-/.](\d{1,2})[-/.](\d{1,2})[-~–—至到]+\s*(\d{4})[-/.](\d{1,2})[-/.](\d{1,2})/
  );
  if (m) {
    return {
      startYear: Number(m[1]),
      startMonth: Number(m[2]),
      startDay: Number(m[3]),
      endMonth: Number(m[5]),
    };
  }
  m = title.match(/(\d{4})[-/.](\d{1,2})[-/.](\d{1,2})[-~–—至到]+\s*(\d{1,2})[-/.](\d{1,2})/);
  if (m) {
    return {
      startYear: Number(m[1]),
      startMonth: Number(m[2]),
      startDay: Number(m[3]),
      endMonth: Number(m[4]),
    };
  }
  return null;
}

/** 扫描全文标题行，提取账本日期范围 */
export function extractYearRangeFromText(text: string): YearRange | null {
  for (const line of text.split(/\r?\n/)) {
    const t = line.trim();
    if (t.startsWith("#")) {
      const yr = extractYearRange(t);
      if (yr) return yr;
    }
  }
  return null;
}

function matchDate(line: string, opts: ParseOptions): { date: string; rest: string } | null {
  // yyyy-mm-dd / yyyy/mm/dd / yyyy.mm.dd
  let m = line.match(/(\d{4})[-/.](\d{1,2})[-/.](\d{1,2})/);
  if (m) return { date: `${m[1]}-${pad(m[2])}-${pad(m[3])}`, rest: line.replace(m[0], " ") };
  // x月x日（年份按标题范围推断，否则当前年）
  m = line.match(/(\d{1,2})月(\d{1,2})[日号]?/);
  if (m) {
    const year = yearForMonth(Number(m[1]), opts);
    return { date: `${year}-${pad(m[1])}-${pad(m[2])}`, rest: line.replace(m[0], " ") };
  }
  // 相对日期（今天/昨天/前天）
  for (const [w, off] of Object.entries(RELATIVE_DAYS)) {
    if (line.includes(w)) return { date: shiftDate(opts.today, off), rest: line.replace(w, " ") };
  }
  // 裸 M-D（年份按标题范围推断，否则当前年）
  m = line.match(/(?<!\d)(\d{1,2})[-/](\d{1,2})(?!\d)/);
  if (m) {
    const year = yearForMonth(Number(m[1]), opts);
    return { date: `${year}-${pad(m[1])}-${pad(m[2])}`, rest: line.replace(m[0], " ") };
  }
  return null;
}

function matchAmount(rest: string): { fen: number; negative: boolean; rest: string } | null {
  const patterns = [
    /[¥￥]\s*(-?\d[\d,]*(?:\.\d{1,2})?)/,
    /(-?\d[\d,]*(?:\.\d{1,2})?)\s*元/,
    /(-?\d[\d,]*\.\d{1,2})/,
    /(?<![\d.])(\d{1,6})(?![\d.])/,
  ];
  for (const re of patterns) {
    const m = rest.match(re);
    if (!m) continue;
    const s = m[1].replace(/,/g, "");
    const negative = s.startsWith("-");
    const v = Math.abs(parseFloat(s));
    if (!Number.isFinite(v) || v <= 0 || v > 99999999) continue;
    return { fen: Math.round(v * 100), negative, rest: rest.replace(m[0], " ") };
  }
  return null;
}

function detectType(line: string): 1 | 2 {
  return INCOME_WORDS.some((w) => line.includes(w)) ? 2 : 1;
}

function matchCategory(rest: string, categories: CategoryRef[]): number | null {
  // 1) 分类名直接命中（名称长的优先，避免误命中）
  const byName = [...categories].sort((a, b) => b.name.length - a.name.length);
  for (const c of byName) {
    if (c.name.length > 0 && rest.includes(c.name)) return c.id;
  }
  // 2) 常见关键词 → 一级分类
  for (const [kw, top] of KEYWORD_MAP) {
    if (rest.includes(kw)) {
      const t = categories.find((c) => c.name === top);
      if (t) return t.id;
    }
  }
  return null;
}

function cleanRemark(rest: string): string {
  let s = rest;
  for (const w of INCOME_WORDS) s = s.replaceAll(w, " ");
  s = s
    .replace(/[，,。;；、\s\-–—:：|｜()（）]+/g, " ")
    .replace(/\s+/g, " ")
    .trim();
  return s;
}

export function parseLine(line: string, opts: ParseOptions): ParsedItem {
  const raw = line.trim();
  const dateMatch = matchDate(raw, opts);
  const rest0 = dateMatch ? dateMatch.rest : raw;
  const amountMatch = matchAmount(rest0);
  const rest1 = amountMatch ? amountMatch.rest : rest0;
  const type = detectType(raw);
  const categoryId = matchCategory(rest1, opts.categories);
  const remark = cleanRemark(rest1);
  const fb = fallbackDate(opts);

  if (!amountMatch) {
    return {
      index: 0,
      raw,
      ok: false,
      warn: "未识别到金额，请补全后勾选导入",
      date: dateMatch?.date ?? fb.date,
      amountFen: 0,
      type,
      categoryId,
      remark,
    };
  }

  return {
    index: 0,
    raw,
    ok: true,
    warn: dateMatch ? "" : fb.warn,
    date: dateMatch?.date ?? fb.date,
    amountFen: amountMatch.fen,
    type: amountMatch.negative ? 1 : type,
    categoryId,
    remark,
  };
}

/** 形如 "5月26日:午饭9.8，晚饭16，水费6，总计" 的「日期:项目金额，项目金额…」格式 → 一行拆多条 */
function expandCommaFormat(line: string, opts: ParseOptions): ParsedItem[] | null {
  const m = line.match(/^([^:：]+)[:：](.+)$/);
  if (!m) return null;
  const dateMatch = matchDate(m[1].trim(), opts);
  if (!dateMatch) return null;
  const parts = m[2].split(/[，,、;；]/).map((s) => s.trim()).filter((s) => s !== "");
  if (parts.length === 0) return null;
  const items: ParsedItem[] = [];
  for (const part of parts) {
    if (/^(总计|合计|小计)/.test(part)) continue;
    const am = matchAmount(part);
    if (!am) continue;
    const name = part
      .replace(/[¥￥]?\s*-?\d[\d,]*(?:\.\d{1,2})?\s*[元块]?/g, " ")
      .replace(/\s+/g, " ")
      .trim();
    items.push({
      index: 0,
      raw: line,
      ok: true,
      warn: "",
      date: dateMatch.date,
      amountFen: am.fen,
      type: am.negative ? 1 : detectType(part),
      categoryId: matchCategory(part, opts.categories),
      remark: name || part,
    });
  }
  return items.length ? items : null;
}

export type ColumnField = "date" | "item" | "amount" | "category" | "remark" | "ignore";

/** 小白友好的自定义规则：选分隔符 + 依次指定每列含义（无需写正则） */
export interface CustomRule {
  name: string;
  separator: "comma" | "space" | "tab" | "pipe";
  columns: ColumnField[];
}

const DATE_SRC = String.raw`\d{4}[-/.]\d{1,2}[-/.]\d{1,2}|\d{1,2}月\d{1,2}日|\d{1,2}[-/]\d{1,2}|今天|昨天|前天`;
const AMOUNT_SRC = String.raw`[¥￥]?\s*-?\d[\d,]*(?:\.\d{1,2})?\s*元?`;
const SEP_SRC: Record<CustomRule["separator"], string> = {
  comma: String.raw`\s*[，,]\s*`,
  space: String.raw`\s+`,
  tab: String.raw`\s*\t\s*`,
  pipe: String.raw`\s*\|\s*`,
};

export function applyCustomRule(
  line: string,
  rule: CustomRule,
  opts: ParseOptions
): ParsedItem | null {
  const parts: string[] = [];
  for (const f of rule.columns) {
    if (f === "date") parts.push(`(?<date>${DATE_SRC})`);
    else if (f === "amount") parts.push(`(?<amount>${AMOUNT_SRC})`);
    else if (f === "item") parts.push("(?<name>.+?)");
    else if (f === "category") parts.push("(?<category>\\S+?)");
    else if (f === "remark") parts.push("(?<remark>.+?)");
    else parts.push(".*?");
  }
  const pattern = parts.join(SEP_SRC[rule.separator] ?? "\\s+");
  let re: RegExp;
  try {
    re = new RegExp(pattern);
  } catch {
    return null;
  }
  const m = line.match(re);
  const g = m?.groups;
  if (!g) return null;
  const dateMatch = g.date ? matchDate(g.date.trim(), opts) : null;
  const am = g.amount ? matchAmount(g.amount) : null;
  if (!am) return null;
  const fb = fallbackDate(opts);
  const type = detectType(g.type ?? line);
  const categoryId = g.category
    ? matchCategory(g.category, opts.categories)
    : matchCategory(line, opts.categories);
  const remark = (g.remark ?? g.name ?? "").trim();
  return {
    index: 0,
    raw: line,
    ok: true,
    warn: dateMatch ? "" : fb.warn,
    date: dateMatch?.date ?? fb.date,
    amountFen: am.fen,
    type: am.negative ? 1 : type,
    categoryId,
    remark,
  };
}

export function parseText(text: string, opts: ParseOptions): ParsedItem[] {
  const lines = text.split(/\r?\n/);
  // 从标题推断账本年份范围（如 # 账本2025.09.01-09.15）
  const yearRange = opts.yearRange ?? extractYearRangeFromText(text);
  const effOpts: ParseOptions = { ...opts, yearRange };

  const items: ParsedItem[] = [];
  let idx = 0;
  for (const line of lines) {
    const t = line.trim();
    if (!t) continue;
    if (t.startsWith("#")) continue; // 跳过 Markdown 标题行
    // 跳过 Markdown 表格的表头与分隔行
    if (t.startsWith("|")) {
      const cells = t.split("|").map((c) => c.trim()).filter((c) => c !== "");
      if (cells.length === 0) continue;
      if (cells.every((c) => /^[-:\s]+$/.test(c))) continue;
      if (cells.some((c) => /^(日期|项目|分类|金额|备注|说明)$/.test(c))) continue;
    }
    // 「日期:项目金额，项目金额…」格式 → 一行拆多条
    const expanded = expandCommaFormat(t, effOpts);
    if (expanded && expanded.length) {
      for (const p of expanded) {
        p.index = ++idx;
        items.push(p);
      }
      continue;
    }
    const item = parseLine(t, effOpts);
    item.index = ++idx;
    items.push(item);
  }
  return items;
}
