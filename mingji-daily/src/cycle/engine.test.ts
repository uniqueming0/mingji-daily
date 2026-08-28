import { describe, expect, it } from "vitest";
import {
  describeRule,
  expandInstances,
  instanceContaining,
  type CycleRule,
} from "./engine";

const base = (over: Partial<CycleRule>): CycleRule => ({
  id: 1,
  name: "t",
  type: "month",
  start_date: "2025-01-01",
  length: 1,
  length_unit: "month",
  week_start: 1,
  month_start_day: 1,
  end_date: null,
  status: 1,
  ...over,
});

describe("周期引擎", () => {
  it("按月 25 日（发薪日周期）", () => {
    const rule = base({ type: "month", month_start_day: 25, start_date: "2025-01-25" });
    const list = expandInstances(rule, "2025-01-20", "2025-03-10");
    expect(list[0]).toMatchObject({ start: "2025-01-25", end: "2025-02-24" });
    expect(list[1]).toMatchObject({ start: "2025-02-25", end: "2025-03-24" });
  });

  it("按月 25 日：月初日期归属上一周期（含头含尾）", () => {
    const rule = base({ type: "month", month_start_day: 25, start_date: "2025-01-25" });
    const inst = instanceContaining(rule, "2025-02-10");
    expect(inst?.start).toBe("2025-01-25");
    expect(inst?.end).toBe("2025-02-24");
  });

  it("按周（周一起始，起始日落在周中）", () => {
    // 2025-08-14 是周四，2025-08-11 是周一
    const rule = base({ type: "week", week_start: 1, start_date: "2025-08-14" });
    const list = expandInstances(rule, "2025-08-14", "2025-08-24");
    expect(list[0]).toMatchObject({ start: "2025-08-11", end: "2025-08-17" });
    expect(list[1]).toMatchObject({ start: "2025-08-18", end: "2025-08-24" });
  });

  it("自定义每 14 天", () => {
    const rule = base({ type: "custom", length: 14, length_unit: "day", start_date: "2025-08-17" });
    const list = expandInstances(rule, "2025-08-17", "2025-09-10");
    expect(list[0]).toMatchObject({ start: "2025-08-17", end: "2025-08-30" });
    expect(list[1]).toMatchObject({ start: "2025-08-31", end: "2025-09-13" });
  });

  it("自定义每 2 个月（跨月钳制）", () => {
    const rule = base({ type: "custom", length: 2, length_unit: "month", start_date: "2025-01-31" });
    const list = expandInstances(rule, "2025-01-01", "2025-04-30");
    expect(list[0]).toMatchObject({ start: "2025-01-31", end: "2025-03-30" });
    expect(list[1]).toMatchObject({ start: "2025-03-31", end: "2025-05-30" });
  });

  it("历史双向展开：规则起始日之前的账单也能归属", () => {
    const rule = base({ type: "month", month_start_day: 1, start_date: "2025-06-01" });
    const inst = instanceContaining(rule, "2025-04-15");
    expect(inst).toMatchObject({ start: "2025-04-01", end: "2025-04-30" });
  });

  it("跨年账本：12 月 20 日起始，1 月 5 日归上年周期", () => {
    const rule = base({ type: "month", month_start_day: 20, start_date: "2025-12-20" });
    const inst = instanceContaining(rule, "2026-01-05");
    expect(inst).toMatchObject({ start: "2025-12-20", end: "2026-01-19" });
  });

  it("结束日期限制", () => {
    const rule = base({ type: "month", month_start_day: 1, start_date: "2025-01-01", end_date: "2025-02-28" });
    const list = expandInstances(rule, "2025-01-01", "2025-06-30");
    expect(list).toHaveLength(2);
    expect(list[1].start).toBe("2025-02-01");
  });

  it("停用规则不产生实例", () => {
    const rule = base({ status: 0 });
    expect(expandInstances(rule, "2025-01-01", "2025-12-31")).toHaveLength(0);
  });

  it("规则描述文案", () => {
    expect(describeRule(base({ type: "week", week_start: 2 }))).toContain("周二");
    expect(describeRule(base({ type: "month", month_start_day: 25 }))).toContain("25 日");
    expect(describeRule(base({ type: "custom", length: 3, length_unit: "week" }))).toContain("每 3 周");
  });
});
