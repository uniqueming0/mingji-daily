<script setup lang="ts">
import { computed, onMounted, ref, watch, type Component } from "vue";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { api } from "./api";
import { appEnv, bootstrap, loadPlatform, navRequest, store, ui } from "./store";
import { toast, toasts } from "./toast";
import { copyText } from "./utils/clipboard";
import RecordView from "./views/RecordView.vue";
import BillsView from "./views/BillsView.vue";
import StatsView from "./views/StatsView.vue";
import ImportView from "./views/ImportView.vue";
import DataView from "./views/DataView.vue";
import SettingsView from "./views/SettingsView.vue";
import HelpView from "./views/HelpView.vue";

interface NavItem {
  name: string;
  icon: string;
  comp: Component;
}

const nav: NavItem[] = [
  { name: "记一笔", icon: "✏️", comp: RecordView },
  { name: "明细", icon: "📋", comp: BillsView },
  { name: "统计", icon: "📊", comp: StatsView },
  { name: "导入", icon: "📥", comp: ImportView },
  { name: "数据管理", icon: "💾", comp: DataView },
  { name: "设置", icon: "⚙️", comp: SettingsView },
  { name: "帮助", icon: "❓", comp: HelpView },
];

const crashReport = ref<string | null>(null);
const updateInfo = ref<{ version: string } | null>(null);
// 页签状态放在 store（ui.active），便于其他页面跳转（如明细页"返回统计"）
const current = computed(() => nav.find((n) => n.name === ui.active)?.comp);

// 统计页穿透 → 切到明细页
watch(
  () => navRequest.seq,
  () => {
    if (navRequest.seq > 0) ui.active = "明细";
  }
);

onMounted(async () => {
  try {
    await bootstrap();
  } catch (e) {
    toast.error(String(e));
  }
  // 检测上次是否异常退出（崩溃报告）
  try {
    crashReport.value = await api.getPendingCrash();
  } catch {
    /* 忽略 */
  }
  // 运行平台（移动端隐藏桌面专属功能）
  await loadPlatform();
  // 静默检查更新（仅桌面端；移动端走应用商店/侧载更新）
  // 注意：启动时没有用户手势，window.confirm 会被 WebView2 静默拒绝，
  // 因此改用应用内自绘弹窗确认
  if (!appEnv.isMobile) {
    try {
      const update = await check();
      if (update) {
        pendingUpdate = update;
        updateInfo.value = { version: update.version };
      }
    } catch {
      /* 便携版或开发环境，忽略 */
    }
  }
});

let pendingUpdate: Update | null = null;

function dismissUpdate() {
  updateInfo.value = null;
  pendingUpdate = null;
}

async function doUpdate() {
  updateInfo.value = null;
  const u = pendingUpdate;
  pendingUpdate = null;
  if (!u) return;
  toast.info("正在下载更新，请稍候…");
  try {
    await u.downloadAndInstall();
  } catch (e) {
    toast.error(String(e));
  }
}

async function copyCrash() {
  const ok = await copyText(crashReport.value ?? "");
  toast.success(ok ? "已复制，请粘贴到反馈群" : "复制失败，请手动全选复制");
}

async function dismissCrash() {
  crashReport.value = null;
  try {
    await api.dismissPendingCrash();
  } catch {
    /* 忽略 */
  }
}
</script>

<template>
  <div class="app">
    <aside class="sidebar">
      <div class="logo">
        <div class="logo-icon">M</div>
        <span>铭记日常</span>
      </div>
      <nav class="nav">
        <button
          v-for="item in nav"
          :key="item.name"
          class="nav-item"
          :class="{ active: ui.active === item.name }"
          @click="ui.active = item.name"
        >
          <span class="icon">{{ item.icon }}</span>{{ item.name }}
        </button>
      </nav>
      <div class="sidebar-footer">v1.3.1 · 数据仅保存在本机</div>
    </aside>

    <main class="content">
      <div v-if="!store.ready" class="loading">加载中…</div>
      <!-- KeepAlive：切换页签时保留各页面状态（如统计页当前选中的周期） -->
      <KeepAlive v-else>
        <component :is="current" />
      </KeepAlive>
    </main>

    <div v-if="crashReport" class="crash-overlay">
      <div class="crash-modal card">
        <h2>😥 检测到上次异常退出</h2>
        <p>点击「复制报告」并把内容发送到反馈群，帮助开发者尽快修复问题。</p>
        <textarea :value="crashReport" readonly rows="6"></textarea>
        <div class="crash-actions">
          <span class="spacer" />
          <button class="btn btn-text" @click="dismissCrash">忽略</button>
          <button class="btn btn-primary" @click="copyCrash">复制报告</button>
        </div>
      </div>
    </div>

    <div v-if="updateInfo" class="crash-overlay">
      <div class="crash-modal card">
        <h2>🎉 发现新版本 v{{ updateInfo.version }}</h2>
        <p>是否现在下载并安装更新？安装完成后应用会自动重启。</p>
        <div class="crash-actions">
          <span class="spacer" />
          <button class="btn btn-text" @click="dismissUpdate">稍后再说</button>
          <button class="btn btn-primary" @click="doUpdate">立即更新</button>
        </div>
      </div>
    </div>

    <div class="toasts">
      <div v-for="t in toasts" :key="t.id" class="toast" :class="t.type">{{ t.text }}</div>
    </div>
  </div>
</template>

<style scoped>
.app {
  display: flex;
  height: 100%;
  background: var(--bg);
}

.sidebar {
  width: 200px;
  flex-shrink: 0;
  background: var(--card);
  border-right: 1px solid var(--line);
  display: flex;
  flex-direction: column;
}

.logo {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 20px 16px;
  font-size: 17px;
  font-weight: 600;
}

.logo-icon {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: var(--primary);
  color: #fff;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
}

.nav {
  flex: 1;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--text-2);
  font-size: 14px;
  text-align: left;
}

.nav-item:hover {
  background: #f0faf5;
}

.nav-item.active {
  background: var(--primary-light);
  color: var(--primary);
  font-weight: 600;
}

.icon {
  font-size: 16px;
}

.sidebar-footer {
  padding: 12px 16px;
  font-size: 12px;
  color: var(--text-3);
}

.content {
  flex: 1;
  padding: 32px;
  overflow: auto;
}

.loading {
  color: var(--text-3);
  padding: 48px 0;
  text-align: center;
}

.toasts {
  position: fixed;
  top: 16px;
  right: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  z-index: 200;
}

.toast {
  padding: 10px 16px;
  border-radius: 8px;
  background: rgba(17, 17, 17, 0.85);
  color: #fff;
  font-size: 13px;
  max-width: 320px;
  border-left: 3px solid var(--primary);
}

.toast.error {
  border-left-color: var(--expense);
}

.toast.info {
  border-left-color: var(--text-3);
}

.crash-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.55);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 300;
}

.crash-modal {
  width: 520px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.crash-modal h2 {
  font-size: 18px;
}

.crash-modal p {
  color: var(--text-2);
  font-size: 13px;
  line-height: 1.7;
}

.crash-modal textarea {
  font-family: Consolas, monospace;
  font-size: 12px;
  border: 1px solid var(--line);
  border-radius: 8px;
  padding: 8px;
  background: var(--bg);
  resize: none;
}

.crash-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* ---------- 移动端布局（Android / iOS）：侧边栏变底部导航 ---------- */
@media (max-width: 820px) {
  .app {
    flex-direction: column;
  }

  /* 侧边栏 → 底部导航栏（7 项平分宽度，无需横向滚动） */
  .sidebar {
    width: 100%;
    order: 2;
    flex-direction: row;
    border-right: none;
    border-top: 1px solid var(--line);
    padding-bottom: env(safe-area-inset-bottom, 0px);
  }

  .logo,
  .sidebar-footer {
    display: none;
  }

  .nav {
    flex-direction: row;
    padding: 6px 4px;
    gap: 0;
  }

  /* 底部导航与系统手势条留出间距（env 安全区在部分 WebView 下为 0，故补固定值） */
  .sidebar {
    padding-bottom: calc(env(safe-area-inset-bottom, 0px) + 8px);
  }

  .nav-item {
    flex: 1;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 3px;
    font-size: 10px;
    padding: 6px 2px;
    border-radius: 10px;
    text-align: center;
    min-width: 0;
    white-space: nowrap;
  }

  .icon {
    font-size: 20px;
    line-height: 1;
  }

  .content {
    order: 1;
    padding: 14px 12px 20px;
    -webkit-overflow-scrolling: touch;
    /* 键盘弹出时留出滚动余量，保证最后一个字段（保存按钮）能滚到可见处 */
    scroll-padding-bottom: 32px;
  }

  /* 提示气泡在手机上贴顶居中 */
  .toasts {
    left: 12px;
    right: 12px;
    align-items: center;
  }
}
</style>
