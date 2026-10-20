<script setup lang="ts">
import { ref } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { api } from "../api";
import { toast } from "../toast";
import type { AiChatMessage, CycleRange } from "../types";

const props = defineProps<{ cycles: CycleRange[] }>();
const emit = defineEmits<{ close: [] }>();

type Tab = "report" | "savings" | "food" | "chat";
const tab = ref<Tab>("report");
const generating = ref(false);
const result = ref<any>(null);
const error = ref("");

// ---------- 自由对话 ----------
const chatMessages = ref<AiChatMessage[]>([]);
const chatInput = ref("");
const chatBusy = ref(false);
const chatBox = ref<HTMLDivElement | null>(null);

const quickAsks = [
  "我这个周期哪方面花得最多？",
  "帮我看看哪里可以少花点",
  "我的饮食支出合理吗？",
  "怎么养成坚持记账的习惯？",
];

async function sendChat() {
  const text = chatInput.value.trim();
  if (!text || chatBusy.value) return;
  chatMessages.value.push({ role: "user", content: text });
  chatInput.value = "";
  chatBusy.value = true;
  try {
    const reply = await api.aiChat(chatMessages.value.slice(-12), props.cycles);
    chatMessages.value.push({ role: "assistant", content: reply || "（无回复）" });
  } catch (e) {
    chatMessages.value.push({ role: "assistant", content: `抱歉，出错了：${String(e)}` });
  } finally {
    chatBusy.value = false;
    // 滚动到底部
    requestAnimationFrame(() => {
      const el = chatBox.value;
      if (el) el.scrollTop = el.scrollHeight;
    });
  }
}

function quickAsk(q: string) {
  chatInput.value = q;
  sendChat();
}

const tabs: Array<{ key: Tab; label: string; icon: string }> = [
  { key: "report", label: "周期报告", icon: "📊" },
  { key: "savings", label: "省钱建议", icon: "💰" },
  { key: "food", label: "饮食推荐", icon: "🍽️" },
  { key: "chat", label: "问 AI", icon: "💬" },
];

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
          v-for="t in tabs"
          :key="t.key"
          class="tab"
          :class="{ active: tab === t.key }"
          @click="tab = t.key"
        >
          {{ t.icon }} {{ t.label }}
        </button>
      </div>

      <button
        v-if="tab !== 'chat'"
        class="btn btn-primary"
        :disabled="generating"
        @click="generate"
      >
        {{ generating ? "生成中，请稍候（约 5~15 秒）…" : "开始生成" }}
      </button>

      <p v-if="error" class="error">⚠️ {{ error }}</p>

      <!-- 自由对话 -->
      <div v-if="tab === 'chat'" class="chat">
        <div ref="chatBox" class="chat-box">
          <div v-if="!chatMessages.length" class="chat-empty">
            <p>💬 可以问我任何关于你记账的问题，例如：</p>
            <div class="quick">
              <button v-for="q in quickAsks" :key="q" class="chip" @click="quickAsk(q)">
                {{ q }}
              </button>
            </div>
          </div>
          <div
            v-for="(m, i) in chatMessages"
            :key="i"
            class="msg"
            :class="m.role === 'user' ? 'mine' : 'ai'"
          >
            <div class="bubble">{{ m.content }}</div>
          </div>
          <div v-if="chatBusy" class="msg ai">
            <div class="bubble typing">思考中…</div>
          </div>
        </div>
        <div class="chat-input">
          <input
            v-model="chatInput"
            type="text"
            placeholder="输入你想问的问题…（回车发送）"
            :disabled="chatBusy"
            @keyup.enter="sendChat"
          />
          <button class="btn btn-primary" :disabled="chatBusy || !chatInput.trim()" @click="sendChat">
            发送
          </button>
        </div>
      </div>

      <div v-if="result && tab !== 'chat'" class="result">
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
            <div class="sug-title">{{ Number(i) + 1 }}. {{ s.title }}</div>
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
            <div class="sug-title">{{ Number(i) + 1 }}. {{ s.item }} → 推荐「{{ s.dish }}」</div>
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

/* ---------- 自由对话 ---------- */
.chat {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.chat-box {
  max-height: 46vh;
  min-height: 220px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 4px 2px;
}

.chat-empty {
  color: var(--text-2);
  font-size: 13px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.quick {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.msg {
  display: flex;
}

.msg.mine {
  justify-content: flex-end;
}

.bubble {
  max-width: 82%;
  padding: 10px 14px;
  border-radius: 14px;
  font-size: 13px;
  line-height: 1.75;
  white-space: pre-wrap;
  word-break: break-word;
}

.msg.mine .bubble {
  background: var(--primary);
  color: #fff;
  border-bottom-right-radius: 4px;
}

.msg.ai .bubble {
  background: var(--bg);
  color: var(--text-1);
  border-bottom-left-radius: 4px;
}

.bubble.typing {
  color: var(--text-3);
}

.chat-input {
  display: flex;
  gap: 8px;
  align-items: center;
}

.chat-input input {
  flex: 1;
}
</style>
