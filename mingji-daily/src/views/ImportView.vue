<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import mammoth from "mammoth";
import { api } from "../api";
import { toast } from "../toast";
import { aiConfig, childrenOf, refreshAiConfig, store, topCategories } from "../store";
import {
  applyCustomRule,
  extractYearRangeFromText,
  parseText,
  type CustomRule,
  type ParsedItem,
  type YearRange,
} from "../import/parser";
import { fenToYuan, todayStr } from "../utils/format";
import type { ConfirmImportInput, ImportTask } from "../types";

const MAX_ROWS = 500;

interface Row {
  checked: boolean;
  raw: string;
  ok: boolean;
  warn: string;
  date: string;
  amountStr: string;
  type: 1 | 2;
  categoryId: number | null;
  remark: string;
}

const mode = ref<"file" | "paste">("file");
const pasteText = ref("");
const fileName = ref("");
const fileMd5 = ref("");
const rows = ref<Row[]>([]);
const parsing = ref(false);
const importing = ref(false);
const tasks = ref<ImportTask[]>([]);

// ---------- 自定义解析规则（最多 3 条，本地持久化） ----------
function loadRules(): CustomRule[] {
  try {
    const arr = JSON.parse(localStorage.getItem("mj:customRules2") || "[]");
    return Array.isArray(arr) ? arr.filter((r) => r && Array.isArray(r.columns)) : [];
  } catch {
    return [];
  }
}
const customRules = ref<CustomRule[]>(loadRules());
const currentYearRange = ref<YearRange | null>(null);

function saveRules() {
  localStorage.setItem("mj:customRules2", JSON.stringify(customRules.value));
}

function addRule() {
  if (customRules.value.length >= 3) {
    toast.error("最多添加 3 条自定义规则");
    return;
  }
  customRules.value.push({
    name: `规则${customRules.value.length + 1}`,
    separator: "comma",
    columns: ["date", "item", "amount", "ignore"],
  });
  saveRules();
}

function removeRule(i: number) {
  customRules.value.splice(i, 1);
  saveRules();
}

function applyRules() {
  if (!customRules.value.length) {
    toast.error("请先添加自定义规则");
    return;
  }
  const opts = {
    today: todayStr(),
    categories: allCategories.value.map((c) => ({ id: c.id, name: c.name })),
    yearRange: currentYearRange.value,
  };
  let fixed = 0;
  for (const r of rows.value) {
    if (r.ok) continue;
    for (const rule of customRules.value) {
      const p = applyCustomRule(r.raw, rule, opts);
      if (p) {
        r.ok = true;
        r.checked = true;
        r.warn = p.warn;
        r.date = p.date;
        r.amountStr = (p.amountFen / 100).toFixed(2);
        r.type = p.type;
        r.categoryId = p.categoryId;
        r.remark = p.remark;
        fixed++;
        break;
      }
    }
  }
  toast.success(fixed ? `自定义规则补全了 ${fixed} 行` : "没有行匹配自定义规则");
}

// ---------- AI 智能识别失败行 ----------
const aiBusy = ref(false);

function findCategoryId(name: string): number | null {
  return store.categories.find((x) => x.name === name && x.status === 1)?.id ?? null;
}

function defaultOtherId(): number | null {
  return store.categories.find((x) => x.name === "其他")?.id ?? null;
}

async function aiParse() {
  const failed = rows.value.filter((r) => !r.ok);
  if (!failed.length) {
    toast.info("没有失败行");
    return;
  }
  aiBusy.value = true;
  try {
    const items = await api.aiParseLines(failed.map((r) => r.raw));
    let fixed = 0;
    for (let i = 0; i < failed.length && i < items.length; i++) {
      const r = failed[i];
      const p = items[i];
      if (!p || !(p.amount > 0)) continue;
      r.date = p.date || r.date;
      r.amountStr = String(p.amount);
      r.type = p.type === 2 ? 2 : 1;
      const cid = findCategoryId(p.category);
      if (cid) r.categoryId = cid;
      if (r.categoryId == null) r.categoryId = defaultOtherId();
      r.remark = p.remark || r.remark;
      r.ok = true;
      r.checked = true;
      r.warn = "AI 识别";
      fixed++;
    }
    toast.success(`AI 补全了 ${fixed} 行，请核对后勾选导入`);
  } catch (e) {
    toast.error(String(e));
  } finally {
    aiBusy.value = false;
  }
}

const tops = computed(() => topCategories());
const allCategories = computed(() => store.categories.filter((c) => c.status === 1));

const checkedCount = computed(() => rows.value.filter((r) => r.checked).length);
const okCount = computed(() => rows.value.filter((r) => r.ok).length);

onMounted(() => {
  loadTasks();
  refreshAiConfig();
});

async function loadTasks() {
  try {
    tasks.value = await api.listImportTasks();
  } catch (e) {
    toast.error(String(e));
  }
}

function toRows(items: ParsedItem[]): Row[] {
  return items.map((it) => ({
    checked: it.ok,
    raw: it.raw,
    ok: it.ok,
    warn: it.warn,
    date: it.date,
    amountStr: (it.amountFen / 100).toFixed(2),
    type: it.type,
    categoryId: it.categoryId,
    remark: it.remark,
  }));
}

function runParse(text: string, name: string, md5: string) {
  parsing.value = true;
  try {
    currentYearRange.value = extractYearRangeFromText(text);
    const items = parseText(text, {
      today: todayStr(),
      categories: allCategories.value.map((c) => ({ id: c.id, name: c.name })),
      yearRange: currentYearRange.value,
    });
    if (items.length === 0) {
      toast.error("未识别到任何账单内容");
      rows.value = [];
      fileName.value = "";
      fileMd5.value = "";
      return;
    }
    if (items.length > MAX_ROWS) {
      toast.info(`内容较多，仅解析前 ${MAX_ROWS} 条，请分批导入`);
    }
    rows.value = toRows(items.slice(0, MAX_ROWS));
    fileName.value = name;
    fileMd5.value = md5;
    toast.success(`识别 ${rows.value.length} 行：${okCount.value} 条可导入，其余请补全`);
  } catch (e) {
    toast.error(String(e));
  } finally {
    parsing.value = false;
  }
}

function base64ToArrayBuffer(b64: string): ArrayBuffer {
  const bin = atob(b64);
  const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
  return bytes.buffer;
}

async function pickFile() {
  try {
    const picked = await open({
      multiple: false,
      filters: [
        { name: "账单文件", extensions: ["txt", "md", "markdown", "docx", "doc"] },
      ],
    });
    if (!picked) return;
    const path = Array.isArray(picked) ? picked[0] : picked;
    parsing.value = true;
    try {
      const file = await api.readImportFile(path);
      // 重复导入提示
      const dup = await api.checkImportDuplicate(file.md5);
      if (dup) {
        const go = confirm(
          `该文件曾于 ${dup.created_at} 导入过（${dup.total} 条），是否继续导入？`
        );
        if (!go) return;
      }
      let text = file.text;
      if (!text && file.base64) {
        const res = await mammoth.extractRawText({
          arrayBuffer: base64ToArrayBuffer(file.base64),
        });
        text = res.value;
      }
      if (!text) {
        toast.error("无法读取文件内容");
        return;
      }
      runParse(text, file.name, file.md5);
    } catch (e) {
      toast.error(String(e));
    } finally {
      parsing.value = false;
    }
  } catch (e) {
    toast.error(String(e));
  }
}

function parsePaste() {
  if (!pasteText.value.trim()) {
    toast.error("请先粘贴账单文本");
    return;
  }
  runParse(pasteText.value, "粘贴文本", "");
}

function checkAll() {
  rows.value.forEach((r) => (r.checked = true));
}

function uncheckAll() {
  rows.value.forEach((r) => (r.checked = false));
}

async function confirmImport() {
  const checked = rows.value.filter((r) => r.checked);
  if (!checked.length) {
    toast.error("请先勾选要导入的账单");
    return;
  }
  const items: ConfirmImportInput["items"] = [];
  for (const r of checked) {
    const fen = Math.round(parseFloat(r.amountStr || "0") * 100);
    if (!(fen > 0)) {
      toast.error(`第 ${rows.value.indexOf(r) + 1} 行金额无效，请检查`);
      return;
    }
    if (!r.categoryId) {
      toast.error(`第 ${rows.value.indexOf(r) + 1} 行未选分类，请检查`);
      return;
    }
    items.push({
      type: r.type,
      amount: fen,
      category_id: r.categoryId,
      bill_date: r.date,
      remark: r.remark.trim(),
    });
  }
  importing.value = true;
  try {
    const task = await api.confirmImport({
      file_name: fileName.value || "粘贴文本",
      file_md5: fileMd5.value,
      items,
    });
    toast.success(`已导入 ${task.total} 条账单`);
    rows.value = [];
    pasteText.value = "";
    fileName.value = "";
    fileMd5.value = "";
    await loadTasks();
  } catch (e) {
    toast.error(String(e));
  } finally {
    importing.value = false;
  }
}

async function revoke(task: ImportTask) {
  if (!confirm(`确定撤销导入「${task.file_name}」吗？其 ${task.total} 条账单将被删除。`)) return;
  try {
    await api.revokeImport(task.id);
    toast.success("已撤销");
    await loadTasks();
  } catch (e) {
    toast.error(String(e));
  }
}
</script>

<template>
  <div class="import-page">
    <h1>导入</h1>

    <div class="mode card">
      <button class="chip" :class="{ active: mode === 'file' }" @click="mode = 'file'">选择文件</button>
      <button class="chip" :class="{ active: mode === 'paste' }" @click="mode = 'paste'">粘贴文本</button>
      <span class="hint">
        支持 txt / md（≤1MB）、docx（≤5MB）；旧版 .doc 请另存为 .docx。
        纯规则解析；AI 语义兜底将在后续版本提供（自备 DeepSeek Key，可开关）。
      </span>
    </div>

    <div v-if="mode === 'file'" class="card">
      <button class="btn btn-primary" :disabled="parsing" @click="pickFile">
        {{ parsing ? "解析中…" : "选择账单文件" }}
      </button>
    </div>

    <div v-else class="card paste-card">
      <textarea
        v-model="pasteText"
        rows="8"
        placeholder="粘贴账单文本，每行一条，例如：&#10;2025-01-15 午餐 25元&#10;2025-01-16 地铁 4元 交通&#10;1月17日 收入 工资 12000元"
      ></textarea>
      <button class="btn btn-primary" :disabled="parsing" @click="parsePaste">解析文本</button>
    </div>

    <div v-if="rows.length" class="preview">
      <div class="pv-head card">
        <span>
          共 {{ rows.length }} 行：可导入 {{ okCount }} 条，待补全
          {{ rows.length - okCount }} 条（默认不勾选）
        </span>
        <span class="spacer" />
        <button class="btn btn-text" @click="checkAll">全选</button>
        <button class="btn btn-text" @click="uncheckAll">全不选</button>
        <button class="btn btn-primary" :disabled="importing" @click="confirmImport">
          确认导入 {{ checkedCount }} 条
        </button>
      </div>

      <div class="table card">
        <div class="t-head">
          <span class="c-check"></span>
          <span class="c-raw">原始内容</span>
          <span class="c-date">日期</span>
          <span class="c-amount">金额</span>
          <span class="c-type">类型</span>
          <span class="c-cat">分类</span>
          <span class="c-remark">备注</span>
        </div>
        <div v-for="(r, i) in rows" :key="i" class="t-row" :class="{ bad: !r.ok }">
          <span class="c-check">
            <input v-model="r.checked" type="checkbox" />
          </span>
          <span class="c-raw" :title="r.raw">{{ r.raw }}</span>
          <span class="c-date">
            <input v-model="r.date" type="text" class="mini-input" />
          </span>
          <span class="c-amount">
            <input v-model="r.amountStr" type="text" inputmode="decimal" class="mini-input" />
          </span>
          <span class="c-type">
            <select v-model="r.type" class="mini-input">
              <option :value="1">支出</option>
              <option :value="2">收入</option>
            </select>
          </span>
          <span class="c-cat">
            <select v-model="r.categoryId" class="mini-input">
              <option :value="null">其他</option>
              <optgroup v-for="t in tops" :key="t.id" :label="t.name">
                <option :value="t.id">{{ t.name }}</option>
                <option v-for="c in childrenOf(t.id)" :key="c.id" :value="c.id">{{ c.name }}</option>
              </optgroup>
            </select>
          </span>
          <span class="c-remark">
            <input v-model="r.remark" type="text" class="mini-input" placeholder="备注" />
          </span>
          <span v-if="r.warn" class="c-warn" :title="r.warn">⚠️</span>
        </div>
      </div>

      <p class="note">
        导入的账单默认计入第一个可用账户，可在明细页修改；待补全行补上金额并勾选后即可一起导入。
      </p>

      <div v-if="okCount < rows.length" class="rules">
        <div class="r-head">
          <span class="r-title">自定义解析规则（最多 3 条，用于解析失败的行）</span>
          <span class="spacer" />
          <button class="btn btn-text" :disabled="customRules.length >= 3" @click="addRule">
            ＋ 添加规则
          </button>
          <button class="btn btn-primary" @click="applyRules">应用规则</button>
          <button
            v-if="aiConfig.enabled"
            class="btn btn-primary"
            :disabled="aiBusy"
            @click="aiParse"
          >
            {{ aiBusy ? "AI 识别中…" : "🤖 AI 识别失败行" }}
          </button>
        </div>
        <div v-for="(rule, i) in customRules" :key="i" class="rule-row">
          <input v-model="rule.name" class="r-name" placeholder="规则名" @change="saveRules" />
          <select v-model="rule.separator" class="r-sep" @change="saveRules">
            <option value="comma">逗号分隔</option>
            <option value="space">空格分隔</option>
            <option value="tab">Tab 分隔</option>
            <option value="pipe">竖线 | 分隔</option>
          </select>
          <select v-for="(col, ci) in rule.columns" :key="ci" v-model="rule.columns[ci]" class="r-col" @change="saveRules">
            <option value="date">日期</option>
            <option value="item">项目名</option>
            <option value="amount">金额</option>
            <option value="category">分类</option>
            <option value="remark">备注</option>
            <option value="ignore">忽略</option>
          </select>
          <button class="btn btn-danger-text" @click="removeRule(i)">删除</button>
        </div>
        <p class="hint">
          小白模式：选择每行的分隔符，再按顺序选择每列的含义（日期 / 项目名 / 金额 / 分类 / 备注 / 忽略），
          无需编写任何代码。<br />
          例如「2025-01-15 午餐 25元」= 空格分隔 + 第1列日期、第2列项目名、第3列金额、第4列忽略。
        </p>
      </div>
    </div>

    <div v-if="tasks.length" class="history card">
      <div class="h-title">最近导入（24 小时内可撤销）</div>
      <div v-for="t in tasks" :key="t.id" class="h-row">
        <span class="h-name">{{ t.file_name }}</span>
        <span class="h-meta">{{ t.created_at }} · {{ t.total }} 条</span>
        <span class="spacer" />
        <button class="btn btn-danger-text" @click="revoke(t)">撤销</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.import-page {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.import-page h1 {
  font-size: 24px;
}

.mode {
  display: flex;
  align-items: center;
  gap: 10px;
}

.hint {
  color: var(--text-3);
  font-size: 12px;
  line-height: 1.6;
}

.paste-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  align-items: flex-start;
}

.paste-card textarea {
  width: 100%;
  resize: vertical;
  line-height: 1.7;
}

.preview {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.pv-head {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 13px;
  color: var(--text-2);
}

.spacer {
  flex: 1;
}

.table {
  padding: 0;
  overflow: auto;
  max-height: 480px;
}

.t-head,
.t-row {
  display: grid;
  grid-template-columns: 32px 1.4fr 110px 90px 70px 130px 1fr 26px;
  gap: 6px;
  align-items: center;
  padding: 6px 12px;
  font-size: 12px;
}

.t-head {
  color: var(--text-3);
  border-bottom: 1px solid var(--line);
  position: sticky;
  top: 0;
  background: var(--card);
}

.t-row {
  border-bottom: 1px solid var(--line);
}

.t-row:last-child {
  border-bottom: none;
}

.t-row.bad {
  background: #fff5f5;
}

.c-raw {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-2);
}

.mini-input {
  width: 100%;
  padding: 4px 6px;
  font-size: 12px;
  border-radius: 6px;
}

.c-warn {
  font-size: 13px;
}

.note {
  font-size: 12px;
  color: var(--text-3);
}

.history {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.h-title {
  font-size: 13px;
  color: var(--text-3);
  margin-bottom: 4px;
}

.h-row {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 13px;
}

.h-name {
  font-weight: 500;
}

.h-meta {
  color: var(--text-3);
  font-size: 12px;
}

.rules {
  display: flex;
  flex-direction: column;
  gap: 8px;
  background: var(--card);
  border-radius: 12px;
  padding: 14px 16px;
}

.r-head {
  display: flex;
  align-items: center;
  gap: 10px;
}

.r-title {
  font-size: 13px;
  color: var(--text-2);
  font-weight: 600;
}

.rule-row {
  display: flex;
  gap: 8px;
}

.r-name {
  width: 130px;
}

.r-sep {
  width: 120px;
}

.r-col {
  width: 96px;
}
</style>
