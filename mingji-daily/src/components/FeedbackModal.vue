<script setup lang="ts">
import { onMounted, ref } from "vue";
import { api } from "../api";
import { toast } from "../toast";
import { copyText } from "../utils/clipboard";

const emit = defineEmits<{ close: [] }>();

const desc = ref("");
const contact = ref("");
const result = ref("");
const appInfo = ref<{ version: string; name: string; os: string } | null>(null);

onMounted(async () => {
  try {
    appInfo.value = await api.getAppInfo();
  } catch {
    /* 忽略 */
  }
});

function generate() {
  if (!desc.value.trim()) {
    toast.error("请先填写问题描述");
    return;
  }
  const a = appInfo.value;
  result.value = [
    "【铭记日常 用户反馈】",
    `版本：${a?.name ?? "铭记日常"} v${a?.version ?? "?"}（${a?.os ?? "Windows"}）`,
    "问题描述：",
    desc.value.trim(),
    contact.value.trim() ? `联系方式：${contact.value.trim()}` : "",
  ]
    .filter(Boolean)
    .join("\n");
}

async function doCopy() {
  if (!result.value) {
    toast.error("请先生成反馈内容");
    return;
  }
  const ok = await copyText(result.value);
  toast.success(ok ? "已复制，请粘贴到反馈群" : "复制失败，请全选后手动复制");
}
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="modal card">
      <h2>意见反馈</h2>
      <label class="field">
        <span>问题描述（必填）</span>
        <textarea v-model="desc" rows="4" placeholder="遇到了什么问题？怎么操作的？"></textarea>
      </label>
      <label class="field">
        <span>联系方式（选填，方便回复您）</span>
        <input v-model="contact" type="text" placeholder="微信号 / 邮箱" />
      </label>
      <button class="btn btn-ghost" @click="generate">生成反馈内容</button>

      <template v-if="result">
        <textarea :value="result" readonly rows="7" class="result"></textarea>
        <div class="actions">
          <span class="hint">复制后粘贴到我们的反馈群即可</span>
          <span class="spacer" />
          <button class="btn btn-text" @click="emit('close')">关闭</button>
          <button class="btn btn-primary" @click="doCopy">复制</button>
        </div>
      </template>
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
  z-index: 150;
}

.modal {
  width: 480px;
  max-height: 88vh;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.modal h2 {
  font-size: 18px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 13px;
  color: var(--text-2);
}

textarea,
input {
  font-family: inherit;
  font-size: 13px;
  border: 1px solid var(--line);
  border-radius: 8px;
  padding: 8px 10px;
  outline: none;
  resize: vertical;
}

textarea:focus,
input:focus {
  border-color: var(--primary);
}

.result {
  background: var(--bg);
  line-height: 1.7;
}

.actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.hint {
  font-size: 12px;
  color: var(--text-3);
}

.spacer {
  flex: 1;
}
</style>
