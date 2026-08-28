<script setup lang="ts">
import { computed, ref } from "vue";
import { api } from "../api";
import { toast } from "../toast";
import { reloadMeta, store } from "../store";
import { describeRule } from "../cycle/engine";
import type { CycleRule } from "../types";
import CycleRuleEditModal from "../components/CycleRuleEditModal.vue";

const showCreate = ref(false);
const editing = ref<CycleRule | null>(null);

const rules = computed(() => store.cycleRules);

async function toggle(r: CycleRule) {
  try {
    await api.updateCycleRule(r.id, {
      name: r.name,
      type: r.type,
      start_date: r.start_date,
      length: r.length,
      length_unit: r.length_unit,
      week_start: r.week_start,
      month_start_day: r.month_start_day,
      end_date: r.end_date,
      status: r.status === 1 ? 0 : 1,
    });
    toast.success(r.status === 1 ? "已停用" : "已启用");
    await reloadMeta();
  } catch (e) {
    toast.error(String(e));
  }
}

async function remove(r: CycleRule) {
  if (!confirm(`确定删除周期规则「${r.name}」吗？（账单数据不受影响）`)) return;
  try {
    await api.deleteCycleRule(r.id);
    toast.success("已删除");
    await reloadMeta();
  } catch (e) {
    toast.error(String(e));
  }
}

async function onSaved() {
  showCreate.value = false;
  editing.value = null;
  await reloadMeta();
}
</script>

<template>
  <div class="cycles">
    <div class="head">
      <p class="hint">
        周期规则决定统计汇总的时间口径；账单按记账日期自动归入所有启用的周期（最多同时启用 5 条）。
      </p>
      <button class="btn btn-primary" @click="showCreate = true">新建规则</button>
    </div>

    <div v-if="!rules.length" class="empty">还没有周期规则，点击「新建规则」创建</div>

    <div v-for="r in rules" :key="r.id" class="card rule" :class="{ off: r.status === 0 }">
      <div class="info">
        <div class="name">{{ r.name }}</div>
        <div class="desc">{{ describeRule(r) }}</div>
      </div>
      <span v-if="r.status === 0" class="badge">已停用</span>
      <span class="spacer" />
      <button class="btn btn-text" @click="editing = r">编辑</button>
      <button class="btn btn-text" @click="toggle(r)">{{ r.status === 1 ? "停用" : "启用" }}</button>
      <button class="btn btn-danger-text" @click="remove(r)">删除</button>
    </div>

    <CycleRuleEditModal v-if="showCreate" :rule="null" @saved="onSaved" @close="showCreate = false" />
    <CycleRuleEditModal v-if="editing" :rule="editing" @saved="onSaved" @close="editing = null" />
  </div>
</template>

<style scoped>
.cycles {
  max-width: 720px;
}

.head {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 14px;
}

.hint {
  flex: 1;
  color: var(--text-3);
  font-size: 12px;
  line-height: 1.6;
}

.empty {
  padding: 32px 0;
  text-align: center;
  color: var(--text-3);
}

.rule {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}

.rule.off {
  opacity: 0.55;
}

.info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.name {
  font-weight: 600;
}

.desc {
  font-size: 12px;
  color: var(--text-3);
}

.badge {
  padding: 2px 8px;
  border-radius: 10px;
  background: #f2f3f5;
  color: var(--text-3);
  font-size: 12px;
}

.spacer {
  flex: 1;
}
</style>
