<script setup lang="ts">
import { onMounted, ref } from "vue";
import { openPath, openUrl } from "@tauri-apps/plugin-opener";
import { check } from "@tauri-apps/plugin-updater";
import { api } from "../api";
import { toast } from "../toast";
import CategoriesView from "./CategoriesView.vue";
import AccountsView from "./AccountsView.vue";
import CycleRulesView from "./CycleRulesView.vue";
import AiSettingsView from "./AiSettingsView.vue";
import FeedbackModal from "../components/FeedbackModal.vue";

const tab = ref<"general" | "categories" | "accounts" | "cycles" | "ai">("general");
const dataDir = ref("");
const mediaUsage = ref<{ count: number; bytes: number } | null>(null);
const showFeedback = ref(false);
const checkingUpdate = ref(false);

onMounted(async () => {
  try {
    dataDir.value = await api.getDataDir();
    mediaUsage.value = await api.getMediaUsage();
  } catch (e) {
    toast.error(String(e));
  }
});

function fmtMB(bytes: number): string {
  return (bytes / 1048576).toFixed(1) + " MB";
}

async function openDataDir() {
  if (!dataDir.value) return;
  try {
    await openPath(dataDir.value);
  } catch (e) {
    toast.error(String(e));
  }
}

async function checkUpdate() {
  checkingUpdate.value = true;
  try {
    const update = await check();
    if (!update) {
      toast.success("已是最新版本");
      return;
    }
    if (!confirm(`发现新版本 v${update.version}，是否下载并安装？`)) return;
    toast.info("正在下载更新，请稍候…");
    await update.downloadAndInstall();
    toast.success("更新已安装，应用即将重启");
  } catch (e) {
    // 便携版不支持自动更新 / 开发环境未签名
    toast.info("当前版本不支持自动更新，将打开下载页");
    try {
      await openUrl("https://github.com/uniqueming0/mingji-daily/releases");
    } catch {
      /* 忽略 */
    }
  } finally {
    checkingUpdate.value = false;
  }
}
</script>

<template>
  <div class="settings">
    <h1>设置</h1>

    <div class="tabs">
      <button class="chip" :class="{ active: tab === 'general' }" @click="tab = 'general'">通用</button>
      <button class="chip" :class="{ active: tab === 'categories' }" @click="tab = 'categories'">分类管理</button>
      <button class="chip" :class="{ active: tab === 'accounts' }" @click="tab = 'accounts'">账户管理</button>
      <button class="chip" :class="{ active: tab === 'cycles' }" @click="tab = 'cycles'">周期管理</button>
      <button class="chip" :class="{ active: tab === 'ai' }" @click="tab = 'ai'">AI 智能服务</button>
    </div>

    <div v-if="tab === 'general'" class="card general">
      <p><b>数据目录：</b>{{ dataDir || "加载中…" }}</p>
      <button class="btn btn-ghost" @click="openDataDir">打开数据目录</button>
      <button class="btn btn-ghost" :disabled="checkingUpdate" @click="checkUpdate">
        {{ checkingUpdate ? "检查中…" : "检查更新" }}
      </button>
      <button class="btn btn-ghost" @click="showFeedback = true">意见反馈</button>
      <p>
        <b>媒体附件：</b>{{
          mediaUsage ? `${mediaUsage.count} 个文件，共 ${fmtMB(mediaUsage.bytes)}` : "加载中…"
        }}
      </p>
      <p class="note">
        账单数据仅保存在本机，默认完全离线，不会上传。<br />
        AI 功能将在后续版本提供（自备 DeepSeek API Key，默认关闭，可随时开启）。
      </p>
    </div>

    <CategoriesView v-else-if="tab === 'categories'" />
    <AccountsView v-else-if="tab === 'accounts'" />
    <CycleRulesView v-else-if="tab === 'cycles'" />
    <AiSettingsView v-else />

    <FeedbackModal v-if="showFeedback" @close="showFeedback = false" />
  </div>
</template>

<style scoped>
.settings h1 {
  font-size: 24px;
  margin-bottom: 16px;
}

.tabs {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
}

.general {
  display: flex;
  flex-direction: column;
  gap: 12px;
  align-items: flex-start;
  max-width: 560px;
}

.note {
  color: var(--text-3);
  font-size: 13px;
  line-height: 1.8;
}
</style>
