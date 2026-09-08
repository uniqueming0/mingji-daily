<script setup lang="ts">
import { ref } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { api } from "../api";
import { toast } from "../toast";
import type { CycleRange } from "../types";

const props = defineProps<{ cycles: CycleRange[] }>();
const emit = defineEmits<{ close: [] }>();

type Tab = "report" | "savings" | "food";
const tab = ref<Tab>("report");
const generating = ref(false);
const result = ref<any>(null);
const error = ref("");

const tabNames: Record<Tab, { label: string; icon: string }> = {
  report: { label: "周期报告", icon: "📊" },
  savings: { label: "省钱建议", icon: "💰" },
  food: { label: "饮食推荐", icon: "🍽️" },
};

async function generate() {
  generating.value = true;
  error.value = "";
  result.value = null;
  try {
    const raw = await api.aiGenerate(tab.value, props.cycles);
    result.value = JSON.parse(raw);
  } catch (e) {
    error.value = String(e);
  } finally {
    generating.value = false;
  }
}

function foodLink(dish: string): string {
  return `https://www.xiachufang.com/search/?keyword=${encodeURIComponent(dish)}`;
}

async function openLink(url: string) {
  try {
    await openUrl(url);
  } catch (e) {
    toast.error(String(e));
  }
}

function feedback(useful: boolean) {
  toast.success(useful ? "感谢反馈：有帮助 👍" : "感谢反馈：会继续优化 🙏");
}
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="modal card">
      <div class="head">
        <h2>✨ AI 分析 <span class="badge">内测</span></h2>
        <button class="btn btn-text" @click="emit('close')">关闭</button>
      </div>

      <div class="tabs">
        <button
          v-for="(t, key) in tabNames"
          :key="key"
          class="tab"
          :class="{ active: tab === key }"
          @click="tab = (key as Tab)"
        >
          {{ t.icon }} {{ t.label }}
        </button>
      </div>

      <button class="btn btn-primary" :disabled="generating" @click="generate">
        {{ generating ? "生成中，请稍候（约 5~15 秒）…" : "开始生成" }}
      </button>

      <p v-if="error" class="error">⚠️ {{ error }}</p>

      <div v-if="result" class="result">
        <!-- 周期报告 -->
        <template v-if="tab === 'report'">
          <h3>{{ result.title }}</h3>
          <ul>
            <li v-for="(p, i) in result.points" :key="i">{{ p }}</li>
          </ul>
          <p class="conclusion">💡 {{ result.conclusion }}</p>
        </template>

        <!-- 省钱建议 -->
        <template v-else-if="tab === 'savings'">
          <div v-for="(s, i) in result.suggestions" :key="i" class="sug">
            <div class="sug-title">{{ i + 1 }}. {{ s.title }}</div>
            <div class="sug-line">📊 依据：{{ s.basis }}</div>
            <div class="sug-line">🛠️ 怎么做：{{ s.action }}</div>
            <div class="sug-line expect">🎯 {{ s.expectation }}</div>
            <div class="fb">
              <button class="btn btn-text" @click="feedback(true)">有用</button>
              <button class="btn btn-text" @click="feedback(false)">没用</button>
            </div>
          </div>
        </template>

        <!-- 饮食推荐 -->
        <template v-else>
          <div v-for="(s, i) in result.suggestions" :key="i" class="sug">
            <div class="sug-title">{{ i + 1 }}. {{ s.item }} → 推荐「{{ s.dish }}」</div>
            <div class="sug-line">📊 依据：{{ s.basis }}</div>
            <div class="sug-line">🍳 {{ s.recipe }}</div>
            <div class="sug-line expect">🎯 {{ s.expectation }}</div>
            <div class="fb">
              <button class="btn btn-ghost small" @click="openLink(foodLink(s.dish))">
                查看菜谱 →
              </button>
              <button class="btn btn-text" @click="feedback(true)">有用</button>
              <button class="btn btn-text" @click="feedback(false)">没用</button>
            </div>
          </div>
        </template>
      </div>

      <p class="disclaimer">
        内测功能，结果由 AI 生成，仅供参考，不构成投资/医疗建议。菜谱外链为第三方网站内容。
      </p>
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
  z-index: 160;
}

.modal {
  width: 560px;
  max-height: 86vh;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.head h2 {
  font-size: 18px;
}

.badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 10px;
  background: var(--primary-light);
  color: var(--primary);
}

.tabs {
  display: flex;
  gap: 8px;
}

.tab {
  flex: 1;
  height: 34px;
  border-radius: 17px;
  border: 1px solid var(--line);
  color: var(--text-2);
  font-size: 13px;
  transition: all 0.15s;
}

.tab.active {
  background: var(--primary);
  border-color: var(--primary);
  color: #fff;
  font-weight: 600;
}

.error {
  color: var(--expense);
  font-size: 13px;
  line-height: 1.7;
}

.result h3 {
  font-size: 15px;
  margin-bottom: 8px;
}

.result ul {
  padding-left: 18px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.result li {
  font-size: 13px;
  color: var(--text-2);
  line-height: 1.7;
}

.conclusion {
  margin-top: 10px;
  font-size: 13px;
  font-weight: 600;
}

.sug {
  border: 1px solid var(--line);
  border-radius: 10px;
  padding: 10px 12px;
  margin-bottom: 8px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.sug-title {
  font-weight: 600;
  font-size: 13px;
}

.sug-line {
  font-size: 12px;
  color: var(--text-2);
  line-height: 1.7;
}

.sug-line.expect {
  color: var(--primary);
  font-weight: 600;
}

.fb {
  display: flex;
  gap: 6px;
  align-items: center;
}

.small {
  height: 28px;
  padding: 0 12px;
  font-size: 12px;
}

.disclaimer {
  font-size: 11px;
  color: var(--text-3);
  line-height: 1.6;
}
</style>
