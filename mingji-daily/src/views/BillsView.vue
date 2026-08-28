<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { api } from "../api";
import { toast } from "../toast";
import { accountName, categoryName, childrenOf, navRequest, store, topCategories } from "../store";
import { currentMonth, fenToYuan, monthRange, shortDate } from "../utils/format";
import type { Bill, BillFilter, BillType } from "../types";
import BillEditModal from "../components/BillEditModal.vue";

const bills = ref<Bill[]>([]);
const loading = ref(false);
const month = ref(currentMonth());
const allTime = ref(false);
const typeFilter = ref<0 | BillType>(0);
const categoryId = ref<number | null>(null);
const accountId = ref<number | null>(null);
const keyword = ref("");
const customRange = ref<{ start: string; end: string } | null>(null);
const editing = ref<Bill | null>(null);

const tops = computed(() => topCategories());
const accounts = computed(() => store.accounts.filter((a) => a.status === 1));

const filter = computed<BillFilter>(() => {
  const f: BillFilter = {};
  if (typeFilter.value) f.type = typeFilter.value;
  if (categoryId.value) f.category_id = categoryId.value;
  if (accountId.value) f.account_id = accountId.value;
  const kw = keyword.value.trim();
  if (kw) f.keyword = kw;
  if (customRange.value) {
    f.start_date = customRange.value.start;
    f.end_date = customRange.value.end;
  } else if (!allTime.value) {
    const r = monthRange(month.value);
    f.start_date = r.start;
    f.end_date = r.end;
  }
  return f;
});

async function load() {
  loading.value = true;
  try {
    bills.value = await api.listBills(filter.value);
  } catch (e) {
    toast.error(String(e));
  } finally {
    loading.value = false;
  }
}

function applyNavFilter() {
  const f = navRequest.filter;
  navRequest.filter = null; // 消费掉，避免之后手动进入明细页时重复套用旧筛选
  if (!f) return;
  customRange.value =
    f.start_date && f.end_date ? { start: f.start_date, end: f.end_date } : null;
  categoryId.value = f.category_id ?? null;
  accountId.value = f.account_id ?? null;
  typeFilter.value = (f.type ?? 0) as 0 | BillType;
  keyword.value = f.keyword ?? "";
}

onMounted(() => {
  // 明细页可能是被统计页"穿透"触发后才挂载的，挂载时先消费一次筛选
  applyNavFilter();
  load();
});
watch([month, allTime, typeFilter, categoryId, accountId, customRange], load);

// 统计页穿透（明细页已挂载时的后续穿透）
watch(
  () => navRequest.seq,
  () => {
    applyNavFilter();
  }
);

const groups = computed(() => {
  const map = new Map<string, Bill[]>();
  for (const b of bills.value) {
    const arr = map.get(b.bill_date);
    if (arr) arr.push(b);
    else map.set(b.bill_date, [b]);
  }
  return Array.from(map.entries());
});

const totals = computed(() => {
  let income = 0;
  let expense = 0;
  for (const b of bills.value) {
    if (b.type === 2) income += b.amount;
    else expense += b.amount;
  }
  return { income, expense, balance: income - expense };
});

function dayExpense(items: Bill[]) {
  return items.filter((b) => b.type === 1).reduce((s, b) => s + b.amount, 0);
}

function dayIncome(items: Bill[]) {
  return items.filter((b) => b.type === 2).reduce((s, b) => s + b.amount, 0);
}

async function onSaved() {
  editing.value = null;
  await load();
}

async function onDeleted() {
  editing.value = null;
  await load();
}
</script>

<template>
  <div class="bills">
    <h1>明细</h1>

    <div class="filters card">
      <div class="seg">
        <button class="seg-item" :class="{ active: typeFilter === 0 }" @click="typeFilter = 0">全部</button>
        <button class="seg-item" :class="{ active: typeFilter === 1 }" @click="typeFilter = 1">支出</button>
        <button class="seg-item" :class="{ active: typeFilter === 2 }" @click="typeFilter = 2">收入</button>
      </div>
      <input v-model="month" type="month" :disabled="allTime || !!customRange" />
      <label class="check"><input v-model="allTime" type="checkbox" /> 不限时间</label>
      <button v-if="customRange" class="btn btn-text" @click="customRange = null">清除日期范围</button>
      <select v-model="categoryId">
        <option :value="null">全部分类</option>
        <optgroup v-for="t in tops" :key="t.id" :label="t.name">
          <option :value="t.id">{{ t.name }}</option>
          <option v-for="c in childrenOf(t.id)" :key="c.id" :value="c.id">{{ c.name }}</option>
        </optgroup>
      </select>
      <select v-model="accountId">
        <option :value="null">全部账户</option>
        <option v-for="a in accounts" :key="a.id" :value="a.id">{{ a.name }}</option>
      </select>
      <div class="search">
        <input v-model="keyword" placeholder="搜索备注" @keyup.enter="load" />
        <button class="btn btn-text" @click="load">搜索</button>
        <button class="btn btn-text" @click="keyword = ''; load()">清除</button>
      </div>
    </div>

    <div class="totals card">
      <span class="income">收入 ¥{{ fenToYuan(totals.income) }}</span>
      <span class="expense">支出 ¥{{ fenToYuan(totals.expense) }}</span>
      <span :class="totals.balance >= 0 ? 'income' : 'expense'">结余 ¥{{ fenToYuan(totals.balance) }}</span>
    </div>

    <div v-if="loading" class="state">加载中…</div>
    <div v-else-if="!groups.length" class="state">暂无账单，去「记一笔」吧</div>
    <div v-else class="groups">
      <div v-for="[date, items] in groups" :key="date" class="group card">
        <div class="group-head">
          <span class="date">{{ shortDate(date) }}</span>
          <span class="day-sum">
            支出 ¥{{ fenToYuan(dayExpense(items)) }} · 收入 ¥{{ fenToYuan(dayIncome(items)) }}
          </span>
        </div>
        <div v-for="b in items" :key="b.id" class="bill-row" @click="editing = b">
          <span class="cat">{{ categoryName(b.category_id) }}</span>
          <span class="remark">{{ b.remark || "—" }}</span>
          <span class="account">{{ accountName(b.account_id) }}</span>
          <span class="amount" :class="b.type === 2 ? 'income' : 'expense'">
            {{ b.type === 2 ? "+" : "-" }}¥{{ fenToYuan(b.amount) }}
          </span>
        </div>
      </div>
    </div>

    <BillEditModal
      v-if="editing"
      :bill="editing"
      @saved="onSaved"
      @deleted="onDeleted"
      @close="editing = null"
    />
  </div>
</template>

<style scoped>
.bills h1 {
  font-size: 24px;
  margin-bottom: 16px;
}

.filters {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
  margin-bottom: 12px;
}

.seg {
  display: flex;
  gap: 6px;
}

.seg-item {
  height: 32px;
  padding: 0 14px;
  border-radius: 16px;
  border: 1px solid var(--line);
  color: var(--text-2);
  transition: all 0.15s;
}

.seg-item.active {
  background: var(--primary);
  border-color: var(--primary);
  color: #fff;
  font-weight: 600;
}

.check {
  display: flex;
  align-items: center;
  gap: 4px;
  color: var(--text-2);
  font-size: 13px;
  white-space: nowrap;
}

.search {
  display: flex;
  align-items: center;
  gap: 4px;
}

.totals {
  display: flex;
  gap: 24px;
  margin-bottom: 12px;
  font-weight: 600;
}

.income {
  color: var(--income);
}

.expense {
  color: var(--expense);
}

.state {
  padding: 48px 0;
  text-align: center;
  color: var(--text-3);
}

.groups {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.group-head {
  display: flex;
  justify-content: space-between;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--line);
  margin-bottom: 4px;
}

.date {
  font-weight: 600;
}

.day-sum {
  color: var(--text-3);
  font-size: 12px;
}

.bill-row {
  display: grid;
  grid-template-columns: 90px 1fr 80px 120px;
  gap: 12px;
  align-items: center;
  padding: 10px 4px;
  border-bottom: 1px solid var(--line);
  cursor: pointer;
  transition: background 0.15s;
}

.bill-row:last-child {
  border-bottom: none;
}

.bill-row:hover {
  background: var(--primary-light);
}

.cat {
  font-weight: 500;
}

.remark {
  color: var(--text-2);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.account {
  color: var(--text-3);
  font-size: 12px;
  text-align: right;
}

.amount {
  text-align: right;
  font-weight: 600;
}
</style>
