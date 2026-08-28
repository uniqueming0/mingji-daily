<script setup lang="ts">
import { computed, reactive, ref } from "vue";
import { api } from "../api";
import { toast } from "../toast";
import { previewInstances } from "../cycle/engine";
import { todayStr } from "../utils/format";
import type { CycleRule, CycleRuleInput, CycleRuleType, LengthUnit } from "../types";

const props = defineProps<{ rule: CycleRule | null }>();
const emit = defineEmits<{ saved: []; close: [] }>();

const draft = reactive({
  name: props.rule?.name ?? "",
  type: (props.rule?.type ?? "month") as CycleRuleType,
  start_date: props.rule?.start_date ?? todayStr(),
  length: props.rule?.length ?? 1,
  length_unit: (props.rule?.length_unit ?? "month") as LengthUnit,
  week_start: props.rule?.week_start ?? 1,
  month_start_day: props.rule?.month_start_day ?? 1,
  end_date: props.rule?.end_date ?? "",
});
const longTerm = ref(props.rule === null || props.rule.end_date === null);
const saving = ref(false);

const dateValid = computed(() => /^\d{4}-\d{2}-\d{2}$/.test(draft.start_date));

const preview = computed(() => {
  if (!dateValid.value) return [];
  try {
    return previewInstances(
      {
        name: draft.name,
        type: draft.type,
        start_date: draft.start_date,
        length: draft.length,
        length_unit: draft.length_unit,
        week_start: draft.week_start,
        month_start_day: draft.month_start_day,
        end_date: longTerm.value ? null : draft.end_date || null,
      },
      3
    );
  } catch {
    return [];
  }
});

async function save() {
  if (!draft.name.trim()) {
    toast.error("请输入规则名称");
    return;
  }
  if (!dateValid.value) {
    toast.error("请选择有效的起始日期");
    return;
  }
  const input: CycleRuleInput = {
    name: draft.name.trim(),
    type: draft.type,
    start_date: draft.start_date,
    length: draft.type === "custom" ? draft.length : 1,
    length_unit: draft.length_unit,
    week_start: draft.week_start,
    month_start_day: draft.month_start_day,
    end_date: longTerm.value ? null : draft.end_date || null,
    status: props.rule ? props.rule.status : 1,
  };
  saving.value = true;
  try {
    if (props.rule) await api.updateCycleRule(props.rule.id, input);
    else await api.createCycleRule(input);
    toast.success("已保存");
    emit("saved");
  } catch (e) {
    toast.error(String(e));
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="modal card">
      <h2>{{ props.rule ? "编辑周期规则" : "新建周期规则" }}</h2>

      <label class="field">
        <span>规则名称</span>
        <input v-model="draft.name" type="text" maxlength="20" placeholder="如：发薪月 / 项目周期" />
      </label>

      <div class="field">
        <span>周期类型</span>
        <div class="seg">
          <button
            type="button"
            class="seg-item"
            :class="{ active: draft.type === 'week' }"
            @click="draft.type = 'week'"
          >
            按周
          </button>
          <button
            type="button"
            class="seg-item"
            :class="{ active: draft.type === 'month' }"
            @click="draft.type = 'month'"
          >
            按月
          </button>
          <button
            type="button"
            class="seg-item"
            :class="{ active: draft.type === 'custom' }"
            @click="draft.type = 'custom'"
          >
            自定义
          </button>
        </div>
      </div>

      <label v-if="draft.type === 'week'" class="field">
        <span>周起始日</span>
        <select v-model="draft.week_start">
          <option v-for="(n, i) in ['周一', '周二', '周三', '周四', '周五', '周六', '周日']" :key="i" :value="i + 1">
            {{ n }}
          </option>
        </select>
      </label>

      <label v-if="draft.type === 'month'" class="field">
        <span>月起始日（1~28 日，短月自动按最后一天顺延）</span>
        <select v-model="draft.month_start_day">
          <option v-for="d in 28" :key="d" :value="d">每月 {{ d }} 日</option>
        </select>
      </label>

      <div v-if="draft.type === 'custom'" class="row2">
        <label class="field">
          <span>周期长度</span>
          <input v-model.number="draft.length" type="number" min="1" />
        </label>
        <label class="field">
          <span>单位</span>
          <select v-model="draft.length_unit">
            <option value="day">天</option>
            <option value="week">周</option>
            <option value="month">个月</option>
          </select>
        </label>
      </div>

      <label class="field">
        <span>起始日期</span>
        <input v-model="draft.start_date" type="date" />
      </label>

      <label class="field">
        <span>结束日期</span>
        <input v-model="draft.end_date" type="date" :disabled="longTerm" />
      </label>
      <label class="check"><input v-model="longTerm" type="checkbox" /> 长期有效</label>

      <div class="preview">
        <div class="pv-title">预览（最近 3 个周期）</div>
        <div v-if="preview.length" class="pv-list">
          <div v-for="p in preview" :key="p.start">{{ p.label }}</div>
        </div>
        <div v-else class="pv-empty">请填写有效的起始日期</div>
      </div>

      <div class="actions">
        <span class="spacer" />
        <button class="btn btn-text" @click="emit('close')">取消</button>
        <button class="btn btn-primary" :disabled="saving" @click="save">保存</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal {
  width: 440px;
  max-height: 86vh;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.modal h2 {
  font-size: 18px;
}

.seg {
  display: flex;
  gap: 8px;
}

.seg-item {
  flex: 1;
  height: 34px;
  border-radius: 17px;
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

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 13px;
  color: var(--text-2);
}

.row2 {
  display: flex;
  gap: 10px;
}

.row2 .field {
  flex: 1;
}

.check {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--text-2);
}

.preview {
  background: var(--bg);
  border-radius: 8px;
  padding: 10px 12px;
}

.pv-title {
  font-size: 12px;
  color: var(--text-3);
  margin-bottom: 6px;
}

.pv-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 13px;
  color: var(--text-1);
}

.pv-empty {
  font-size: 13px;
  color: var(--text-3);
}

.actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.spacer {
  flex: 1;
}
</style>
