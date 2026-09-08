<script setup lang="ts">
import { onMounted, ref } from "vue";
import { api } from "../api";
import { toast } from "../toast";
import { aiConfig, refreshAiConfig } from "../store";

const keyInput = ref("");
const keySaved = ref(false);
const testing = ref(false);
const testResult = ref("");

onMounted(async () => {
  await refreshAiConfig();
  keySaved.value = aiConfig.hasKey;
});

async function toggleEnabled() {
  try {
    const cfg = await api.setAiConfig({ enabled: !aiConfig.enabled });
    aiConfig.enabled = cfg.enabled;
    toast.success(cfg.enabled ? "AI 功能已开启" : "AI 功能已关闭");
  } catch (e) {
    toast.error(String(e));
  }
}

async function saveKey() {
  if (!keyInput.value.trim()) {
    toast.error("请先粘贴 API Key");
    return;
  }
  try {
    const cfg = await api.setAiConfig({ apiKey: keyInput.value.trim() });
    aiConfig.hasKey = cfg.hasKey;
    keyInput.value = "";
    keySaved.value = cfg.hasKey;
    toast.success("Key 已保存（仅存本机）");
  } catch (e) {
    toast.error(String(e));
  }
}

async function clearKey() {
  if (!confirm("确定删除已保存的 API Key 吗？")) return;
  try {
    const cfg = await api.setAiConfig({ apiKey: "" });
    aiConfig.hasKey = cfg.hasKey;
    keySaved.value = false;
    toast.success("Key 已删除");
  } catch (e) {
    toast.error(String(e));
  }
}

async function testConn() {
  testing.value = true;
  testResult.value = "";
  try {
    testResult.value = await api.testAiConnection();
  } catch (e) {
    testResult.value = "失败：" + String(e);
  } finally {
    testing.value = false;
  }
}
</script>

<template>
  <div class="ai-settings">
    <div class="card">
      <div class="row">
        <div>
          <div class="title">AI 智能服务 <span class="badge">内测</span></div>
          <div class="desc">智能识别导入账单 + 省钱建议 / 饮食推荐 / 周期报告（自备 DeepSeek Key）</div>
        </div>
        <button class="switch" :class="{ on: aiConfig.enabled }" @click="toggleEnabled">
          <span class="dot"></span>
        </button>
      </div>
      <div v-if="!aiConfig.enabled" class="hint">当前已关闭，开启后即可使用全部 AI 功能。</div>
    </div>

    <div class="card">
      <div class="title">API Key（仅保存在本机）</div>
      <div v-if="keySaved" class="row">
        <span class="hint">✅ 已保存 Key（{{ aiConfig.model }}）</span>
        <button class="btn btn-ghost" :disabled="testing" @click="testConn">
          {{ testing ? "测试中…" : "测试连接" }}
        </button>
        <button class="btn btn-danger-text" @click="clearKey">删除 Key</button>
      </div>
      <template v-else>
        <input v-model="keyInput" type="password" placeholder="sk-..." class="key-input" />
        <button class="btn btn-primary" @click="saveKey">保存 Key</button>
      </template>
      <p v-if="testResult" class="hint">{{ testResult }}</p>
    </div>

    <details class="card tutorial">
      <summary>📖 如何获取 DeepSeek API Key？（1 分钟）</summary>
      <ol class="steps">
        <li>浏览器打开 <b>platform.deepseek.com</b>，用手机号/微信注册登录</li>
        <li>左侧「API Keys」→ 点「创建 API Key」→ 复制生成的 <code>sk-xxxx</code></li>
        <li>回到本页面，粘贴 → 保存 Key → 点「测试连接」</li>
        <li>打开总开关，即可在「导入」页用 AI 识别失败行、在「统计」页用 AI 分析</li>
      </ol>
      <p class="hint">费用：DeepSeek 按用量收费（本软件用量很小，日常约几毛钱/月）。</p>
    </details>

    <div class="card">
      <div class="title">隐私说明</div>
      <p class="hint">
        Key 只保存在您的电脑上，不会上传给开发者。<br />
        开启 AI 后：导入识别会发送「识别失败的行文本」；AI 分析只发送「分类金额等聚合统计」，
        不会发送账单备注等原始内容。数据仅用于生成您本人的分析结果，不用于训练。
      </p>
    </div>
  </div>
</template>

<style scoped>
.ai-settings {
  display: flex;
  flex-direction: column;
  gap: 12px;
  max-width: 640px;
}

.card {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.title {
  font-weight: 600;
  font-size: 14px;
}

.desc {
  font-size: 12px;
  color: var(--text-3);
  margin-top: 4px;
}

.badge {
  padding: 2px 8px;
  border-radius: 10px;
  background: var(--primary-light);
  color: var(--primary);
  font-size: 11px;
}

.hint {
  font-size: 12px;
  color: var(--text-3);
  line-height: 1.7;
}

.switch {
  margin-left: auto;
  width: 46px;
  height: 26px;
  border-radius: 13px;
  background: var(--text-4);
  position: relative;
  transition: background 0.2s;
  flex-shrink: 0;
}

.switch.on {
  background: var(--primary);
}

.switch .dot {
  position: absolute;
  top: 3px;
  left: 3px;
  width: 20px;
  height: 20px;
  border-radius: 10px;
  background: #fff;
  transition: left 0.2s;
}

.switch.on .dot {
  left: 23px;
}

.key-input {
  font-family: Consolas, monospace;
}

.steps {
  padding-left: 20px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.steps li {
  font-size: 13px;
  color: var(--text-2);
  line-height: 1.7;
}

code {
  background: var(--bg);
  padding: 1px 6px;
  border-radius: 4px;
  font-family: Consolas, monospace;
  font-size: 12px;
}
</style>
