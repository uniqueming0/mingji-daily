<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import * as echarts from "echarts";
import { api } from "../api";
import { toast } from "../toast";
import { categoryName, gotoBills, store } from "../store";
import { addMonthsClamped, expandInstances, instanceContaining } from "../cycle/engine";
import { fenToYuan, todayStr } from "../utils/format";
import type { Bill, CycleInstance, CycleRule } from "../types";

// ---------- 周期切换 ----------
const ruleId = ref<number | null>(null);
const enabledRules = computed(() => store.cycleRules.filter((r) => r.status === 1));
const rule = computed<CycleRule | null>(
  () => enabledRules.value.find((r) => r.id === ruleId.value) ?? enabledRules.value[0] ?? null
);

const curIndex = ref(0);

const instanceList = computed<CycleInstance[]>(() => {
  if (!rule.value) return [];
  const cur = instanceContaining(rule.value, todayStr());
  const anchor = cur?.start ?? rule.value.start_date;
  return expandInstances(rule.value, addMonthsClamped(anchor, -24), addMonthsClamped(anchor, 24));
});

const inst = computed<CycleInstance | null>(() => instanceList.value[curIndex.value] ?? null);
const prevInst = computed<CycleInstance | null>(() => {
  const i = curIndex.value - 1;
  return i >= 0 ? instanceList.value[i] : null;
});

function goToToday() {
  if (!rule.value) return;
  const cur = instanceContaining(rule.value, todayStr());
  const idx = cur
    ? instanceList.value.findIndex((i) => i.start === cur.start)
    : instanceList.value.length - 1;
  curIndex.value = idx;
}
// 注意：必须先注册 goToToday，再注册 load 监听（首次立即执行顺序依赖）
watch(rule, goToToday, { immediate: true });

// ---------- 数据加载 ----------
const bills = ref<Bill[]>([]);
const prevBills = ref<Bill[]>([]);
const loading = ref(false);

async function load() {
  if (!inst.value) {
    bills.value = [];
    prevBills.value = [];
    return;
  }
  loading.value = true;
  try {
    const [cur, prev] = await Promise.all([
      api.listBills({ start_date: inst.value.start, end_date: inst.value.end }),
      prevInst.value
        ? api.listBills({ start_date: prevInst.value.start, end_date: prevInst.value.end })
        : Promise.resolve([] as Bill[]),
    ]);
    bills.value = cur;
    prevBills.value = prev;
  } catch (e) {
    toast.error(String(e));
  } finally {
    loading.value = false;
  }
}

watch([inst, prevInst], load, { immediate: true });

// ---------- 汇总计算 ----------
function summarize(arr: Bill[]) {
  let income = 0;
  let expense = 0;
  const days = new Set<string>();
  for (const b of arr) {
    if (b.type === 2) income += b.amount;
    else expense += b.amount;
    days.add(b.bill_date);
  }
  return { income, expense, balance: income - expense, recordedDays: days.size };
}

const summary = computed(() => summarize(bills.value));
const prevSummary = computed(() => summarize(prevBills.value));

function dateDiffDays(start: string, end: string): number {
  return Math.round((new Date(end).getTime() - new Date(start).getTime()) / 86400000) + 1;
}

const totalDays = computed(() => (inst.value ? dateDiffDays(inst.value.start, inst.value.end) : 0));
const avgDaily = computed(() =>
  totalDays.value ? Math.round(summary.value.expense / totalDays.value) : 0
);

function ratio(cur: number, prev: number): number | null {
  if (prev === 0) return null;
  return Math.round(((cur - prev) / prev) * 1000) / 10;
}

function fmtRatio(r: number | null): string {
  if (r === null) return "—";
  return r >= 0 ? `+${r}%` : `${r}%`;
}

function pct(part: number, total: number): number {
  return total ? Math.round((part / total) * 1000) / 10 : 0;
}

// ---------- 分类汇总 ----------
interface CatStat {
  id: number;
  name: string;
  amount: number;
}

const level = ref<"top" | "leaf">("top");

const expenseByCategory = computed<CatStat[]>(() => {
  const map = new Map<number, number>();
  for (const b of bills.value) {
    if (b.type !== 1) continue;
    let id = b.category_id;
    if (level.value === "top") {
      const cat = store.categories.find((c) => c.id === b.category_id);
      if (cat?.parent_id != null) id = cat.parent_id;
    }
    map.set(id, (map.get(id) ?? 0) + b.amount);
  }
  return Array.from(map.entries())
    .map(([id, amount]) => ({ id, name: categoryName(id), amount }))
    .sort((a, b) => b.amount - a.amount);
});

const incomeByCategory = computed<CatStat[]>(() => {
  const map = new Map<number, number>();
  for (const b of bills.value) {
    if (b.type !== 2) continue;
    let id = b.category_id;
    const cat = store.categories.find((c) => c.id === b.category_id);
    if (cat?.parent_id != null) id = cat.parent_id;
    map.set(id, (map.get(id) ?? 0) + b.amount);
  }
  return Array.from(map.entries())
    .map(([id, amount]) => ({ id, name: categoryName(id), amount }))
    .sort((a, b) => b.amount - a.amount);
});

// ---------- 趋势数据 ----------
const trendData = computed(() => {
  if (!inst.value) return { labels: [] as string[], values: [] as number[], avg: 0 };
  const useMonth = totalDays.value > 120;
  const map = new Map<string, number>();
  for (const b of bills.value) {
    if (b.type !== 1) continue;
    const key = useMonth ? b.bill_date.slice(0, 7) : b.bill_date;
    map.set(key, (map.get(key) ?? 0) + b.amount);
  }
  const labels: string[] = [];
  const values: number[] = [];
  if (useMonth) {
    let m = inst.value.start.slice(0, 7);
    const endM = inst.value.end.slice(0, 7);
    while (m <= endM && labels.length < 60) {
      labels.push(m);
      values.push(Math.round((map.get(m) ?? 0) / 100));
      const [y, mo] = m.split("-").map(Number);
      const nd = new Date(y, mo, 1);
      m = `${nd.getFullYear()}-${String(nd.getMonth() + 1).padStart(2, "0")}`;
    }
  } else {
    let d = inst.value.start;
    while (d <= inst.value.end && labels.length < 500) {
      labels.push(d.slice(5));
      values.push(Math.round((map.get(d) ?? 0) / 100));
      const nd = new Date(d);
      nd.setDate(nd.getDate() + 1);
      d = `${nd.getFullYear()}-${String(nd.getMonth() + 1).padStart(2, "0")}-${String(
        nd.getDate()
      ).padStart(2, "0")}`;
    }
  }
  const avg = values.length ? values.reduce((s, v) => s + v, 0) / values.length : 0;
  return { labels, values, avg };
});

// ---------- 图表 ----------
const pieEl = ref<HTMLDivElement | null>(null);
const trendEl = ref<HTMLDivElement | null>(null);
let pieChart: echarts.ECharts | null = null;
let pieDom: HTMLElement | null = null;
let trendChart: echarts.ECharts | null = null;

function renderPie() {
  // 容器不存在（本期无支出，v-if 移除）→ 释放图表
  if (!pieEl.value) {
    if (pieChart) {
      pieChart.dispose();
      pieChart = null;
      pieDom = null;
    }
    return;
  }
  // 容器节点变化（空 → 有数据重新渲染出新 div）→ 在旧节点上的图表已失效，释放
  if (pieDom !== pieEl.value) {
    pieChart?.dispose();
    pieChart = null;
    pieDom = null;
  }
  // 保证 pieChart 非空（TS 收窄）
  if (!pieChart) {
    pieChart = echarts.init(pieEl.value);
    pieDom = pieEl.value;
  }
  const data = expenseByCategory.value.map((c) => ({
    name: c.name,
    value: Math.round(c.amount / 100),
  }));
  pieChart.setOption({
    tooltip: { trigger: "item", formatter: "{b}: ¥{c} ({d}%)" },
    color: ["#00b578", "#3b7cff", "#f7b500", "#f5483b", "#8e6bf0", "#14b8c9", "#f2789f", "#94a3b8"],
    series: [
      {
        type: "pie",
        radius: ["42%", "70%"],
        center: ["50%", "50%"],
        label: { show: false },
        data,
      },
    ],
  });
}

function renderTrend() {
  if (!trendEl.value) return;
  if (!trendChart) trendChart = echarts.init(trendEl.value);
  const { labels, values, avg } = trendData.value;
  trendChart.setOption({
    tooltip: { trigger: "axis" },
    grid: { left: 56, right: 16, top: 28, bottom: 36 },
    xAxis: { type: "category", data: labels, axisLabel: { fontSize: 10 } },
    yAxis: { type: "value", name: "元" },
    series: [
      {
        type: "bar",
        data: values,
        itemStyle: { color: "#00b578" },
        barMaxWidth: 18,
        markLine: {
          silent: true,
          symbol: "none",
          lineStyle: { color: "#f5483b", type: "dashed" },
          label: { formatter: `日均 ¥${avg.toFixed(0)}`, color: "#f5483b" },
          data: [{ yAxis: avg }],
        },
      },
    ],
  });
}

function onResize() {
  pieChart?.resize();
  trendChart?.resize();
}

// flush: "post"：等 DOM 更新（v-if 的图表容器创建出来）之后再渲染图表
watch(expenseByCategory, renderPie, { flush: "post" });
watch(trendData, renderTrend, { flush: "post" });
onMounted(() => {
  renderPie();
  renderTrend();
  window.addEventListener("resize", onResize);
});
onBeforeUnmount(() => {
  window.removeEventListener("resize", onResize);
  pieChart?.dispose();
  trendChart?.dispose();
});

// ---------- 明细穿透 ----------
function drill(c: CatStat, billType: 1 | 2) {
  if (!inst.value) return;
  gotoBills({
    category_id: c.id,
    type: billType,
    start_date: inst.value.start,
    end_date: inst.value.end,
  });
}
</script>

<template>
  <div class="stats">
    <div class="head">
      <h1>统计</h1>
      <div v-if="rule" class="switcher">
        <select v-model="ruleId">
          <option v-for="r in enabledRules" :key="r.id" :value="r.id">{{ r.name }}</option>
        </select>
        <button class="btn btn-ghost small" :disabled="curIndex <= 0" @click="curIndex--">
          ◀ 上一周期
        </button>
        <span class="range">{{ inst?.label }}</span>
        <button
          class="btn btn-ghost small"
          :disabled="curIndex >= instanceList.length - 1"
          @click="curIndex++"
        >
          下一周期 ▶
        </button>
        <button class="btn btn-text" @click="goToToday">回到本期</button>
      </div>
    </div>

    <div v-if="loading" class="loading">加载中…</div>

    <div v-if="!rule" class="empty">暂无启用的周期规则，请到「设置 → 周期管理」创建或启用</div>

    <template v-else-if="inst">
      <div class="cards">
        <div class="stat-card card">
          <div class="v income">¥{{ fenToYuan(summary.income) }}</div>
          <div class="k">收入</div>
        </div>
        <div class="stat-card card">
          <div class="v expense">¥{{ fenToYuan(summary.expense) }}</div>
          <div class="k">支出</div>
        </div>
        <div class="stat-card card">
          <div class="v" :class="summary.balance >= 0 ? 'income' : 'expense'">
            ¥{{ fenToYuan(summary.balance) }}
          </div>
          <div class="k">结余</div>
        </div>
        <div class="stat-card card">
          <div class="v">¥{{ fenToYuan(avgDaily) }}</div>
          <div class="k">日均支出</div>
        </div>
        <div class="stat-card card">
          <div class="v">{{ summary.recordedDays }}<span class="unit">/{{ totalDays }} 天</span></div>
          <div class="k">记账天数</div>
        </div>
      </div>

      <div class="grid2">
        <div class="card">
          <div class="sec-head">
            <h2>支出分类占比</h2>
            <div class="seg">
              <button class="mini" :class="{ active: level === 'top' }" @click="level = 'top'">一级</button>
              <button class="mini" :class="{ active: level === 'leaf' }" @click="level = 'leaf'">二级</button>
            </div>
          </div>
          <div v-if="expenseByCategory.length" class="pie-row">
            <div ref="pieEl" class="pie"></div>
            <div class="rank">
              <div
                v-for="(c, i) in expenseByCategory.slice(0, 8)"
                :key="c.id"
                class="rank-row"
                @click="drill(c, 1)"
              >
                <span class="idx">{{ i + 1 }}</span>
                <span class="cname">{{ c.name }}</span>
                <span class="camount">¥{{ fenToYuan(c.amount) }}</span>
                <span class="cpct">{{ pct(c.amount, summary.expense) }}%</span>
              </div>
              <div v-if="expenseByCategory.length > 8" class="more">
                其余 {{ expenseByCategory.length - 8 }} 个分类未展示
              </div>
            </div>
          </div>
          <div v-else class="empty">本期暂无支出</div>
        </div>

        <div class="card">
          <div class="sec-head">
            <h2>收入分布</h2>
          </div>
          <div v-if="incomeByCategory.length" class="rank">
            <div
              v-for="(c, i) in incomeByCategory"
              :key="c.id"
              class="rank-row"
              @click="drill(c, 2)"
            >
              <span class="idx">{{ i + 1 }}</span>
              <span class="cname">{{ c.name }}</span>
              <span class="camount income">¥{{ fenToYuan(c.amount) }}</span>
              <span class="cpct">{{ pct(c.amount, summary.income) }}%</span>
            </div>
          </div>
          <div v-else class="empty">本期暂无收入</div>
        </div>
      </div>

      <div class="card trend-card">
        <div class="sec-head">
          <h2>周期内{{ totalDays > 120 ? "月度" : "每日" }}支出趋势</h2>
          <span class="hint">红色虚线为日均支出</span>
        </div>
        <div ref="trendEl" class="trend"></div>
      </div>

      <div v-if="prevInst" class="card">
        <div class="sec-head">
          <h2>周期对比（上一周期 {{ prevInst.label }}）</h2>
        </div>
        <div class="cmp-row">
          <span class="cmp-k">总支出</span>
          <span class="cmp-v">¥{{ fenToYuan(prevSummary.expense) }} → ¥{{ fenToYuan(summary.expense) }}</span>
          <span class="cmp-r" :class="ratio(summary.expense, prevSummary.expense) !== null && ratio(summary.expense, prevSummary.expense)! > 0 ? 'bad' : 'good'">
            {{ fmtRatio(ratio(summary.expense, prevSummary.expense)) }}
          </span>
        </div>
        <div class="cmp-row">
          <span class="cmp-k">总收入</span>
          <span class="cmp-v">¥{{ fenToYuan(prevSummary.income) }} → ¥{{ fenToYuan(summary.income) }}</span>
          <span class="cmp-r" :class="ratio(summary.income, prevSummary.income) !== null && ratio(summary.income, prevSummary.income)! < 0 ? 'bad' : 'good'">
            {{ fmtRatio(ratio(summary.income, prevSummary.income)) }}
          </span>
        </div>
        <div class="cmp-row">
          <span class="cmp-k">结余</span>
          <span class="cmp-v">¥{{ fenToYuan(prevSummary.balance) }} → ¥{{ fenToYuan(summary.balance) }}</span>
          <span class="cmp-r" :class="ratio(summary.balance, prevSummary.balance) !== null && ratio(summary.balance, prevSummary.balance)! < 0 ? 'bad' : 'good'">
            {{ fmtRatio(ratio(summary.balance, prevSummary.balance)) }}
          </span>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.stats {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 12px;
}

.head h1 {
  font-size: 24px;
}

.switcher {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.small {
  height: 30px;
  padding: 0 12px;
  font-size: 13px;
}

.range {
  font-weight: 600;
  color: var(--text-2);
  min-width: 200px;
  text-align: center;
}

.loading {
  color: var(--text-3);
  padding: 24px 0;
  text-align: center;
}

.empty {
  color: var(--text-3);
  padding: 32px 0;
  text-align: center;
}

.cards {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 12px;
}

.stat-card .v {
  font-size: 22px;
  font-weight: 700;
}

.stat-card .v .unit {
  font-size: 12px;
  font-weight: 400;
  color: var(--text-3);
}

.stat-card .k {
  font-size: 12px;
  color: var(--text-3);
  margin-top: 4px;
}

.income {
  color: var(--income);
}

.expense {
  color: var(--expense);
}

.grid2 {
  display: grid;
  grid-template-columns: 3fr 2fr;
  gap: 14px;
}

.sec-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.sec-head h2 {
  font-size: 16px;
}

.hint {
  font-size: 12px;
  color: var(--text-3);
}

.seg {
  display: flex;
  gap: 6px;
}

.mini {
  height: 26px;
  padding: 0 12px;
  border-radius: 13px;
  border: 1px solid var(--line);
  color: var(--text-2);
  font-size: 12px;
  transition: all 0.15s;
}

.mini.active {
  background: var(--primary-light);
  border-color: var(--primary);
  color: var(--primary);
  font-weight: 600;
}

.pie-row {
  display: flex;
  gap: 16px;
  align-items: center;
}

.pie {
  width: 250px;
  height: 250px;
  flex-shrink: 0;
}

.rank {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.rank-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 7px 8px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.15s;
}

.rank-row:hover {
  background: var(--primary-light);
}

.idx {
  width: 20px;
  height: 20px;
  border-radius: 10px;
  background: var(--bg);
  color: var(--text-3);
  font-size: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.cname {
  font-weight: 500;
}

.camount {
  margin-left: auto;
  font-weight: 600;
}

.cpct {
  width: 52px;
  text-align: right;
  color: var(--text-3);
  font-size: 12px;
}

.more {
  font-size: 12px;
  color: var(--text-3);
  text-align: center;
  padding: 4px 0;
}

.trend-card {
  padding-bottom: 8px;
}

.trend {
  width: 100%;
  height: 280px;
}

.cmp-row {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 8px 0;
  border-bottom: 1px solid var(--line);
}

.cmp-row:last-child {
  border-bottom: none;
}

.cmp-k {
  width: 64px;
  color: var(--text-2);
}

.cmp-v {
  flex: 1;
  font-weight: 600;
}

.cmp-r {
  font-weight: 700;
}

.cmp-r.good {
  color: var(--income);
}

.cmp-r.bad {
  color: var(--expense);
}
</style>
