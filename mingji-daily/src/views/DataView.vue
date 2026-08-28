<script setup lang="ts">
import { onMounted, ref } from "vue";
import { open, save } from "@tauri-apps/plugin-dialog";
import { api } from "../api";
import { toast } from "../toast";
import type { BackupInfo } from "../types";

const integrity = ref("");
const checking = ref(false);
const backups = ref<BackupInfo[]>([]);
const busy = ref(false);
const orphan = ref<{ count: number; bytes: number } | null>(null);

onMounted(loadBackups);

function fmtMB(bytes: number): string {
  return (bytes / 1048576).toFixed(1) + " MB";
}

async function loadBackups() {
  try {
    backups.value = await api.listBackups();
  } catch (e) {
    toast.error(String(e));
  }
}

async function checkIntegrity() {
  checking.value = true;
  try {
    integrity.value = await api.checkDbIntegrity();
  } catch (e) {
    toast.error(String(e));
  } finally {
    checking.value = false;
  }
}

async function doBackup(withMedia: boolean) {
  busy.value = true;
  try {
    const b = await api.createBackup(withMedia);
    toast.success(`备份完成：${b.name}`);
    await loadBackups();
  } catch (e) {
    toast.error(String(e));
  } finally {
    busy.value = false;
  }
}

async function doRestore(b: BackupInfo) {
  const msg = b.with_media
    ? `确定从备份「${b.name}」恢复吗？当前数据库与媒体文件将被替换。`
    : `确定从备份「${b.name}」恢复吗？当前数据库将被替换。`;
  if (!confirm(msg)) return;
  busy.value = true;
  try {
    await api.restoreBackup(b.name);
    toast.success("恢复成功，请重启应用以加载恢复后的数据");
  } catch (e) {
    toast.error(String(e));
  } finally {
    busy.value = false;
  }
}

async function doDelete(b: BackupInfo) {
  if (!confirm(`确定删除备份「${b.name}」吗？`)) return;
  try {
    await api.deleteBackup(b.name);
    toast.success("已删除");
    await loadBackups();
  } catch (e) {
    toast.error(String(e));
  }
}

async function exportCsv() {
  try {
    const path = await save({
      defaultPath: "铭记日常-账单.csv",
      filters: [{ name: "CSV", extensions: ["csv"] }],
    });
    if (!path) return;
    const r = await api.exportBillsCsv(path);
    toast.success(`已导出 ${r.count} 条账单`);
  } catch (e) {
    toast.error(String(e));
  }
}

async function exportJson() {
  try {
    const path = await save({
      defaultPath: "铭记日常-全部数据.json",
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (!path) return;
    const r = await api.exportAllJson(path);
    toast.success(`已导出 ${r.count} 条账单及全部配置`);
  } catch (e) {
    toast.error(String(e));
  }
}

async function checkOrphan() {
  try {
    const o = await api.getOrphanMedia();
    orphan.value = o;
    if (!o.count) toast.success("没有孤儿文件");
  } catch (e) {
    toast.error(String(e));
  }
}

async function cleanOrphan() {
  if (!confirm("确定清理孤儿媒体文件吗？（未被任何账单引用的文件将被删除）")) return;
  try {
    const o = await api.cleanOrphanMedia();
    orphan.value = { count: 0, bytes: 0 };
    toast.success(`已清理 ${o.count} 个文件，释放 ${fmtMB(o.bytes)}`);
  } catch (e) {
    toast.error(String(e));
  }
}

async function migrateDir() {
  try {
    const dir = await open({ directory: true, multiple: false, title: "选择新的数据目录" });
    if (!dir) return;
    const target = Array.isArray(dir) ? dir[0] : dir;
    if (!confirm(`确定将数据迁移到「${target}」吗？迁移完成后请重启应用。`)) return;
    const r = await api.migrateDataDir(target);
    toast.success(`已迁移到 ${r}，请重启应用（旧数据目录请手动删除）`);
  } catch (e) {
    toast.error(String(e));
  }
}
</script>

<template>
  <div class="data-page">
    <h1>数据管理</h1>

    <div class="card">
      <div class="sec-head">
        <h2>数据库状态</h2>
        <button class="btn btn-ghost" :disabled="checking" @click="checkIntegrity">
          {{ checking ? "检查中…" : "检查完整性" }}
        </button>
      </div>
      <p v-if="integrity">
        <span v-if="integrity === 'ok'" class="ok">✅ 数据库正常</span>
        <span v-else class="bad">⚠️ 检查结果：{{ integrity }} —— 建议立即从备份恢复</span>
      </p>
      <p v-else class="hint">定期检查数据库完整性，异常时请从备份恢复。</p>
    </div>

    <div class="card">
      <div class="sec-head">
        <h2>备份</h2>
        <span class="hint">每次退出应用自动备份（仅数据），保留最近 7 份</span>
      </div>
      <div class="btn-row">
        <button class="btn btn-primary" :disabled="busy" @click="doBackup(false)">
          立即备份（仅数据）
        </button>
        <button class="btn btn-ghost" :disabled="busy" @click="doBackup(true)">
          立即备份（含照片/视频）
        </button>
      </div>

      <div v-if="backups.length" class="b-list">
        <div v-for="b in backups" :key="b.name" class="b-row">
          <span class="b-name">{{ b.name }}</span>
          <span class="b-meta">{{ b.created_at }} · {{ fmtMB(b.size) }}</span>
          <span v-if="b.with_media" class="badge">含媒体</span>
          <span class="spacer" />
          <button class="btn btn-text" :disabled="busy" @click="doRestore(b)">恢复</button>
          <button class="btn btn-danger-text" @click="doDelete(b)">删除</button>
        </div>
      </div>
      <p v-else class="hint">暂无备份。恢复备份后请重启应用。</p>
    </div>

    <div class="card">
      <div class="sec-head"><h2>导出</h2></div>
      <div class="btn-row">
        <button class="btn btn-primary" @click="exportCsv">导出账单 CSV</button>
        <button class="btn btn-ghost" @click="exportJson">导出全部数据 JSON（含分类/周期/媒体清单）</button>
      </div>
      <p class="hint">CSV 可直接用 Excel 打开（UTF-8 BOM）；JSON 用于迁移到未来的小程序版或换机。</p>
    </div>

    <div class="card">
      <div class="sec-head"><h2>孤儿媒体文件</h2></div>
      <div class="btn-row">
        <button class="btn btn-ghost" @click="checkOrphan">检查</button>
        <button class="btn btn-danger-ghost" @click="cleanOrphan">清理</button>
      </div>
      <p v-if="orphan" class="hint">
        {{ orphan.count ? `发现 ${orphan.count} 个孤儿文件，共 ${fmtMB(orphan.bytes)}` : "没有孤儿文件" }}
      </p>
      <p v-else class="hint">孤儿文件 = 未被任何账单引用的媒体文件（异常残留），可安全清理。</p>
    </div>

    <div class="card">
      <div class="sec-head"><h2>数据目录迁移</h2></div>
      <div class="btn-row">
        <button class="btn btn-ghost" @click="migrateDir">迁移到新目录</button>
      </div>
      <p class="hint">
        用于便携版 ↔ 安装版切换或搬家：选择目标文件夹后自动复制数据库与媒体，重启应用生效（旧目录请手动删除）。
      </p>
    </div>
  </div>
</template>

<style scoped>
.data-page {
  display: flex;
  flex-direction: column;
  gap: 14px;
  max-width: 760px;
}

.data-page h1 {
  font-size: 24px;
}

.sec-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 12px;
}

.sec-head h2 {
  font-size: 16px;
}

.btn-row {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.hint {
  color: var(--text-3);
  font-size: 12px;
  line-height: 1.7;
  margin-top: 10px;
}

.ok {
  color: var(--income);
  font-weight: 600;
}

.bad {
  color: var(--expense);
  font-weight: 600;
}

.b-list {
  margin-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.b-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 0;
  border-bottom: 1px solid var(--line);
  font-size: 13px;
}

.b-row:last-child {
  border-bottom: none;
}

.b-name {
  font-weight: 500;
}

.b-meta {
  color: var(--text-3);
  font-size: 12px;
}

.badge {
  padding: 2px 8px;
  border-radius: 10px;
  background: var(--primary-light);
  color: var(--primary);
  font-size: 12px;
}

.spacer {
  flex: 1;
}
</style>
