import { describe, expect, it } from "vitest";
import {
  applyCustomRule,
  extractYearRangeFromText,
  parseLine,
  parseText,
  type ParseOptions,
} from "./parser";

const cats = [
  { id: 1, name: "餐饮" },
  { id: 2, name: "午餐" },
  { id: 3, name: "交通" },
  { id: 4, name: "地铁" },
  { id: 5, name: "居住" },
  { id: 6, name: "娱乐" },
  { id: 7, name: "收入类" },
  { id: 8, name: "工资" },
];
const opts: ParseOptions = { today: "2026-08-19", categories: cats };

describe("导入解析引擎", () => {
  it("回归：2025-01-15 不能解析成 2025-001-15", () => {
    const r = parseLine("2025-01-15 午餐 25元", opts);
    expect(r.date).toBe("2025-01-15");
    expect(r.amountFen).toBe(2500);
    expect(r.categoryId).toBe(2);
  });

  it("点号与斜杠日期", () => {
    expect(parseLine("2025.09.01 早饭5", opts).date).toBe("2025-09-01");
    expect(parseLine("2025/9/3 地铁 12", opts).date).toBe("2025-09-03");
  });

  it("x月x日 与相对日期", () => {
    expect(parseLine("9月1日 午饭 10", opts).date).toBe("2026-09-01");
    expect(parseLine("昨天 聚餐 128", opts).date).toBe("2026-08-18");
    expect(parseLine("前天 打车 20", opts).date).toBe("2026-08-17");
  });

  it("收入关键词与负金额", () => {
    expect(parseLine("2025-01-17 收入 工资 12000元", opts).type).toBe(2);
    expect(parseLine("2025-01-17 房租 -2500", opts).type).toBe(1);
    expect(parseLine("2025-01-17 房租 -2500", opts).amountFen).toBe(250000);
  });

  it("一行多笔格式（日期:项目金额，…，总计）", () => {
    const text = "9月1日:早饭5，午饭10，晚饭16，总计";
    const items = parseText(text, {
      ...opts,
      yearRange: { startYear: 2025, startMonth: 9, startDay: 1, endMonth: 9 },
    });
    expect(items).toHaveLength(3);
    expect(items[0]).toMatchObject({ date: "2025-09-01", amountFen: 500 });
    expect(items[2].amountFen).toBe(1600);
  });

  it("标题年份推断", () => {
    const text = "# 账本2025.09.01-09.15\n9月1日:午饭10，总计";
    const items = parseText(text, opts);
    expect(items[0].date).toBe("2025-09-01");
    expect(extractYearRangeFromText(text)?.startYear).toBe(2025);
  });

  it("跨年标题：起始月之前的月份归次年", () => {
    const text = "# 账本2025.12.20-2026.01.10\n1月5日:午饭10，总计";
    const items = parseText(text, opts);
    expect(items[0].date).toBe("2026-01-05");
  });

  it("无日期行默认账本起始日", () => {
    const text = "# 账本2025.09.01-09.15\n收入1300";
    const items = parseText(text, opts);
    expect(items[0].date).toBe("2025-09-01");
    expect(items[0].type).toBe(2);
    expect(items[0].warn).toContain("账本起始日");
  });

  it("Markdown 标题与表格头跳过", () => {
    const text = "# 标题\n| 日期 | 项目 | 金额 |\n| --- | --- | --- |\n2025-01-15 午餐 25元";
    const items = parseText(text, opts);
    expect(items).toHaveLength(1);
  });

  it("自定义列规则（小白模式：竖线分隔）", () => {
    const r = applyCustomRule(
      "2025-01-15|午餐|25",
      { name: "t", separator: "pipe", columns: ["date", "item", "amount"] },
      opts
    );
    expect(r?.date).toBe("2025-01-15");
    expect(r?.amountFen).toBe(2500);
    expect(r?.remark).toContain("午餐");
  });

  it("自定义列规则（空格分隔）", () => {
    const r = applyCustomRule(
      "2025-01-15 午餐 25元",
      { name: "t", separator: "space", columns: ["date", "item", "amount"] },
      opts
    );
    expect(r?.amountFen).toBe(2500);
  });
});
