<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { api, mediaSrc } from "../api";
import { toast } from "../toast";
import { childrenOf, store, topCategories } from "../store";
import { isPhoto, isVideo } from "../utils/media";
import type { Bill, BillType, Media } from "../types";

const props = defineProps<{ bill: Bill }>();
const emit = defineEmits<{ saved: []; deleted: []; close: [] }>();

const type = ref<BillType>(props.bill.type);
const amountStr = ref((props.bill.amount / 100).toFixed(2));
const categoryId = ref(props.bill.category_id);
const accountId = ref(props.bill.account_id);
const billDate = ref(props.bill.bill_date);
const remark = ref(props.bill.remark);
const saving = ref(false);
const deleting = ref(false);

// 媒体附件
const media = ref<Media[]>([]);
const mediaBusy = ref(false);
const lightbox = ref<Media | null>(null);

const tops = computed(() => topCategories());
const accounts = computed(() => store.accounts.filter((a) => a.status === 1));

async function loadMedia() {
  try {
    media.value = await api.listMedia(props.bill.id);
  } catch (e) {
    toast.error(String(e));
  }
}
onMounted(loadMedia);

async function pickAndAttach() {
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

    // 预校验：与账单上已附加的媒体合计，超限则整批拦截（不附加任何文件）
    let photoCount = media.value.filter((m) => m.type === 1).length;
    let videoCount = media.value.filter((m) => m.type === 2).length;
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

    mediaBusy.value = true;
    for (const p of list) {
      await api.attachMedia(props.bill.id, p);
    }
    await loadMedia();
  } catch (e) {
    toast.error(String(e));
  } finally {
    mediaBusy.value = false;
  }
}

async function removeMedia(m: Media) {
  if (!confirm("确定删除这个附件吗？")) return;
  try {
    await api.deleteMedia(m.id);
    media.value = media.value.filter((x) => x.id !== m.id);
    toast.success("已删除");
  } catch (e) {
    toast.error(String(e));
  }
}

async function save() {
  const fen = Math.round(parseFloat(amountStr.value || "0") * 100);
  if (!(fen > 0)) {
    toast.error("请输入正确的金额");
    return;
  }
  saving.value = true;
  try {
    await api.updateBill(props.bill.id, {
      type: type.value,
      amount: fen,
      category_id: categoryId.value,
      account_id: accountId.value,
      bill_date: billDate.value,
      remark: remark.value.trim(),
    });
    toast.success("已保存");
    emit("saved");
  } catch (e) {
    toast.error(String(e));
  } finally {
    saving.value = false;
  }
}

async function del() {
  if (!confirm("确定删除这笔账单吗？（其照片/视频将一并删除）")) return;
  deleting.value = true;
  try {
    await api.deleteBill(props.bill.id);
    toast.success("已删除");
    emit("deleted");
  } catch (e) {
    toast.error(String(e));
  } finally {
    deleting.value = false;
  }
}
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="modal card">
      <h2>编辑账单</h2>

      <div class="seg">
        <button type="button" class="seg-item" :class="{ active: type === 1 }" @click="type = 1">支出</button>
        <button type="button" class="seg-item" :class="{ active: type === 2 }" @click="type = 2">收入</button>
      </div>

      <label class="field">
        <span>金额</span>
        <input v-model="amountStr" type="text" inputmode="decimal" placeholder="0.00" />
      </label>

      <label class="field">
        <span>分类</span>
        <select v-model="categoryId">
          <optgroup v-for="t in tops" :key="t.id" :label="t.name">
            <option :value="t.id">{{ t.name }}</option>
            <option v-for="c in childrenOf(t.id)" :key="c.id" :value="c.id">{{ c.name }}</option>
          </optgroup>
        </select>
      </label>

      <label class="field">
        <span>账户</span>
        <select v-model="accountId">
          <option v-for="a in accounts" :key="a.id" :value="a.id">{{ a.name }}</option>
        </select>
      </label>

      <label class="field">
        <span>日期</span>
        <input v-model="billDate" type="date" />
      </label>

      <label class="field">
        <span>备注</span>
        <input v-model="remark" type="text" maxlength="100" placeholder="选填" />
      </label>

      <div class="field">
        <span>照片 / 视频（{{ media.length }}，照片最多 9 张、视频最多 1 个）</span>
        <div v-if="media.length" class="media-grid">
          <div v-for="m in media" :key="m.id" class="media-item">
            <img
              v-if="m.type === 1"
              :src="mediaSrc(m.thumb_path ?? m.path)"
              class="thumb"
              alt="照片"
              @click="lightbox = m"
            />
            <button v-else class="video-tile" @click="lightbox = m">
              🎬<span class="vname">{{ m.file_name }}</span>
            </button>
            <button class="del" title="删除附件" @click="removeMedia(m)">✕</button>
          </div>
        </div>
        <button type="button" class="btn btn-ghost" :disabled="mediaBusy" @click="pickAndAttach">
          ＋ 添加照片 / 视频
        </button>
      </div>

      <div class="actions">
        <button class="btn btn-danger-ghost" :disabled="deleting" @click="del">删除</button>
        <span class="spacer" />
        <button class="btn btn-text" @click="emit('close')">取消</button>
        <button class="btn btn-primary" :disabled="saving" @click="save">保存</button>
      </div>
    </div>

    <div v-if="lightbox" class="lightbox" @click.self="lightbox = null">
      <img v-if="lightbox.type === 1" :src="mediaSrc(lightbox.path)" alt="预览" />
      <video v-else :src="mediaSrc(lightbox.path)" controls autoplay />
      <button class="lb-close" @click="lightbox = null">✕</button>
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
  width: 460px;
  max-height: 88vh;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 14px;
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

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 13px;
  color: var(--text-2);
}

.media-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
}

.media-item {
  position: relative;
  aspect-ratio: 1;
  border-radius: 8px;
  overflow: hidden;
  border: 1px solid var(--line);
  background: var(--bg);
}

.thumb {
  width: 100%;
  height: 100%;
  object-fit: cover;
  cursor: zoom-in;
}

.video-tile {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  font-size: 22px;
}

.vname {
  font-size: 11px;
  color: var(--text-3);
  max-width: 90%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.del {
  position: absolute;
  top: 4px;
  right: 4px;
  width: 20px;
  height: 20px;
  border-radius: 10px;
  background: rgba(0, 0, 0, 0.55);
  color: #fff;
  font-size: 11px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.del:hover {
  background: var(--expense);
}

.lightbox {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.85);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
}

.lightbox img,
.lightbox video {
  max-width: 90vw;
  max-height: 86vh;
  border-radius: 8px;
}

.lb-close {
  position: absolute;
  top: 20px;
  right: 24px;
  width: 36px;
  height: 36px;
  border-radius: 18px;
  background: rgba(255, 255, 255, 0.2);
  color: #fff;
  font-size: 16px;
}

.actions {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 4px;
}

.spacer {
  flex: 1;
}
</style>
