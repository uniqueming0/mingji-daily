function pad(n: number): string {
  return n < 10 ? "0" + n : "" + n;
}

/** 分 → 元字符串（千分位 + 两位小数） */
export function fenToYuan(fen: number): string {
  return (fen / 100).toLocaleString("zh-CN", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
}

export function todayStr(): string {
  const d = new Date();
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

export function currentMonth(): string {
  const d = new Date();
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}`;
}

/** "2025-06" → { start: "2025-06-01", end: "2025-06-30" } */
export function monthRange(month: string): { start: string; end: string } {
  const [y, m] = month.split("-").map(Number);
  const last = new Date(y, m, 0).getDate();
  return { start: `${y}-${pad(m)}-01`, end: `${y}-${pad(m)}-${pad(last)}` };
}

/** "2025-06-15" → "6月15日" */
export function shortDate(date: string): string {
  const [, m, d] = date.split("-").map(Number);
  return `${m}月${d}日`;
}
