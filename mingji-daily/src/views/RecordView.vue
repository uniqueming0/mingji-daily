<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { api } from "../api";
import { toast } from "../toast";
import { childrenOf, store, topCategories } from "../store";
import { todayStr } from "../utils/format";
import { isPhoto, isVideo } from "../utils/media";
import type { BillType, Category } from "../types";

const type = ref<BillType>(Number(localStorage.getItem("mj:lastType") || 1) as BillType);
const amountStr = ref("");
const selectedTop = ref<number | null>(Number(localStorage.getItem("mj:lastTop")) || null);
const selected = ref<number | null>(Number(localStorage.getItem("mj:lastCategory")) || null);
const accountId = ref<number>(Number(localStorage.getItem("mj:lastAccount")) || 0);
const billDate = ref(todayStr());
const remark = ref("");
const saving = ref(false);
const pendingFiles = ref<{ path: string; name: string }[]>([]);
const amountInput = ref<HTMLInputElement | null>(null);

const tops = computed(() => topCategories());
const children = computed(() => (selectedTop.value != null ? childrenOf(selectedTop.value) : []));
const accounts = computed(() => store.accounts.filter((a) => a.status === 1));

onMounted(() => {
  if (selectedTop.value == null) selectedTop.value = tops.value[0]?.id ?? null;
  if (selected.value == null && selectedTop.value != null) {
    const kids = childrenOf(selectedTop.value);
    selected.value = kids.length ? kids[0].id : selectedTop.value;
  }
  if (!accountId.value) accountId.value = accounts.value[0]?.id ?? 0;
  amountInput.value?.focus();
});

function pickTop(id: number) {
  selectedTop.value = id;
  const kids = childrenOf(id);
  selected.value = kids.length ? kids[0].id : id;
}

function pickChild(c: Category) {
  selected.value = c.id;
}

async function pickFiles() {
  try {
    const picked = await open({
      multiple: true,
      filters: [
        { name: "照片", extensions: ["jpg", "jpeg", "png", "gif", "webp", "bmp"] },
        { name: "视频", extensions: ["mp4", "webm", "mov", "m4v"] },
      ],
    });
    if (!picked) return;
    const list = Array.isArray(picked) ? picked : [picked];

    // 预校验：与已排队附件合计，超限则整批拦截（不加入任何文件）
    let photoCount = pendingFiles.value.filter((f) => isPhoto(f.path)).length;
    let videoCount = pendingFiles.value.filter((f) => isVideo(f.path)).length;
    for (const p of list) {
      if (isPhoto(p)) photoCount++;
      else if (isVideo(p)) videoCount++;
    }
    if (photoCount > 9) {
      toast.error(`照片最多 9 张（当前已选 ${photoCount} 张），请删除多余照片后再添加`);
      return;
    }
    if (videoCount > 1) {
      toast.error(`视频最多 1 个（当前已选 ${videoCount} 个），请删除多余视频后再添加`);
      return;
    }

    for (const p of list) {
      pendingFiles.value.push({ path: p, name: p.split(/[\\/]/).pop() ?? p });
    }
  } catch (e) {
    toast.error(String(e));
  }
}

function removePending(i: number) {
  pendingFiles.value.splice(i, 1);
}

async function save() {
  const fen = Math.round(parseFloat(amountStr.value || "0") * 100);
  if (!(fen > 0)) {
    toast.error("请输入正确的金额");
    amountInput.value?.focus();
    return;
  }
  if (selected.value == null) {
    toast.error("请选择分类");
    return;
  }
  if (!accountId.value) {
    toast.error("请选择账户");
    return;
  }
  // 保存前兜底校验：超限则整体拒绝，不做部分上传
  const pCount = pendingFiles.value.filter((f) => isPhoto(f.path)).length;
  const vCount = pendingFiles.value.filter((f) => isVideo(f.path)).length;
  if (pCount > 9) {
    toast.error("照片最多 9 张，请先删除多余照片再保存");
    return;
  }
  if (vCount > 1) {
    toast.error("视频最多 1 个，请先删除多余视频再保存");
    return;
  }
  saving.value = true;
  try {
    const bill = await api.createBill({
      type: type.value,
      amount: fen,
      category_id: selected.value,
      account_id: accountId.value,
      bill_date: billDate.value,
      remark: remark.value.trim(),
    });
    toast.success("已记一笔");
    localStorage.setItem("mj:lastType", String(type.value));
    localStorage.setItem("mj:lastCategory", String(selected.value));
    localStorage.setItem("mj:lastAccount", String(accountId.value));
    localStorage.setItem("mj:lastTop", String(selectedTop.value));
    // 附件：逐张/逐段附加到新账单
    const pending = pendingFiles.value.splice(0);
    for (const f of pending) {
      try {
        await api.attachMedia(bill.id, f.path);
      } catch (e) {
        toast.error(`附件失败：${f.name}（${String(e)}）`);
      }
    }
    if (pending.length) toast.info(`已添加 ${pending.length} 个附件`);
    amountStr.value = "";
    remark.value = "";
    amountInput.value?.focus();
  } catch (e) {
    toast.error(String(e));
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <div class="record">
    <h1>记一笔</h1>
    <form class="card form" @submit.prevent="save">
      <div class="seg">
        <button type="button" class="seg-item" :class="{ active: type === 1 }" @click="type = 1">支出</button>
        <button type="button" class="seg-item" :class="{ active: type === 2 }" @click="type = 2">收入</button>
      </div>

      <div class="amount-row">
        <span class="currency">¥</span>
        <input
          ref="amountInput"
          v-model="amountStr"
          class="amount-input"
          type="text"
          inputmode="decimal"
          placeholder="0.00"
        />
      </div>

      <div class="field">
        <div class="label">分类</div>
        <div class="chips">
          <button
            v-for="t in tops"
            :key="t.id"
            type="button"
            class="chip"
            :class="{ active: t.id === selectedTop }"
            @click="pickTop(t.id)"
          >
            {{ t.name }}
          </button>
        </div>
        <div v-if="children.length" class="cat-grid">
          <button
            v-for="c in children"
            :key="c.id"
            type="button"
            class="cat-cell"
            :class="{ active: c.id === selected }"
            @click="pickChild(c)"
          >
            {{ c.name }}
          </button>
        </div>
        <div v-else class="hint">
          当前分类：{{ tops.find((t) => t.id === selectedTop)?.name ?? "—" }}（无子分类，点击上方标签即选中）
        </div>
      </div>

      <div class="row">
        <div class="field grow">
          <div class="label">账户</div>
          <div class="chips">
            <button
              v-for="a in accounts"
              :key="a.id"
              type="button"
              class="chip"
              :class="{ active: a.id === accountId }"
              @click="accountId = a.id"
            >
              {{ a.name }}
            </button>
          </div>
        </div>
        <div class="field">
          <div class="label">日期</div>
          <input v-model="billDate" type="date" />
        </div>
      </div>

      <div class="field">
        <div class="label">备注</div>
        <input v-model="remark" type="text" maxlength="100" placeholder="选填，例如：和朋友聚餐" />
      </div>

      <div class="field">
        <div class="label">照片 / 视频（选填，照片最多 9 张、视频最多 1 个）</div>
        <div v-if="pendingFiles.length" class="chips">
          <span v-for="(f, i) in pendingFiles" :key="i" class="file-chip">
            {{ f.name }}
            <button type="button" class="x" @click="removePending(i)">✕</button>
          </span>
        </div>
        <button type="button" class="btn btn-ghost attach" @click="pickFiles">📎 添加照片 / 视频</button>
      </div>

      <button class="btn btn-primary submit" type="submit" :disabled="saving">
        {{ saving ? "保存中…" : "保存" }}
      </button>
    </form>
  </div>
</template>

<style scoped>
.record h1 {
  font-size: 24px;
  margin-bottom: 16px;
}

.form {
  display: flex;
  flex-direction: column;
  gap: 18px;
  max-width: 640px;
}

.seg {
  display: flex;
  gap: 8px;
}

.seg-item {
  flex: 1;
  max-width: 160px;
  height: 36px;
  border-radius: 18px;
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

.amount-row {
  display: flex;
  align-items: center;
  gap: 8px;
  border-bottom: 2px solid var(--primary);
  padding: 4px 0;
}

.currency {
  font-size: 28px;
  color: var(--primary);
  font-weight: 600;
}

.amount-input {
  flex: 1;
  border: none;
  font-size: 32px;
  font-weight: 700;
  padding: 4px 0;
  color: var(--text-1);
}

.amount-input:focus {
  border: none;
  box-shadow: none;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.label {
  font-size: 13px;
  color: var(--text-2);
}

.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.cat-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(88px, 1fr));
  gap: 8px;
  margin-top: 8px;
}

.cat-cell {
  padding: 10px 8px;
  border-radius: 10px;
  background: #fff;
  border: 1px solid var(--line);
  font-size: 13px;
  color: var(--text-2);
  transition: all 0.15s;
}

.cat-cell:hover {
  border-color: var(--primary);
  color: var(--primary);
}

.cat-cell.active {
  background: var(--primary-light);
  border-color: var(--primary);
  color: var(--primary);
  font-weight: 600;
}

.hint {
  font-size: 12px;
  color: var(--text-3);
}

.row {
  display: flex;
  gap: 16px;
  align-items: flex-start;
}

.grow {
  flex: 1;
}

.submit {
  align-self: flex-start;
  min-width: 160px;
}

.file-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  border-radius: 12px;
  background: var(--primary-light);
  color: var(--text-2);
  font-size: 12px;
  max-width: 260px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-chip .x {
  color: var(--text-3);
  font-size: 12px;
}

.file-chip .x:hover {
  color: var(--expense);
}

.attach {
  align-self: flex-start;
}
</style>
