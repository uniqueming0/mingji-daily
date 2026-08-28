<script setup lang="ts">
import { ref } from "vue";
import { api } from "../api";
import { toast } from "../toast";
import { reloadMeta, store } from "../store";
import type { Account } from "../types";

const newName = ref("");
const editingId = ref<number | null>(null);
const editingName = ref("");

async function add() {
  const name = newName.value.trim();
  if (!name) return;
  try {
    await api.createAccount(name);
    toast.success("已添加");
    newName.value = "";
    await reloadMeta();
  } catch (e) {
    toast.error(String(e));
  }
}

function startEdit(a: Account) {
  editingId.value = a.id;
  editingName.value = a.name;
}

async function saveEdit() {
  if (editingId.value == null) return;
  const name = editingName.value.trim();
  if (!name) return;
  const cur = store.accounts.find((a) => a.id === editingId.value);
  if (!cur) return;
  try {
    await api.updateAccount(editingId.value, name, cur.status);
    toast.success("已保存");
    editingId.value = null;
    await reloadMeta();
  } catch (e) {
    toast.error(String(e));
  }
}

async function toggle(a: Account) {
  try {
    await api.updateAccount(a.id, a.name, a.status === 1 ? 0 : 1);
    toast.success(a.status === 1 ? "已停用" : "已启用");
    await reloadMeta();
  } catch (e) {
    toast.error(String(e));
  }
}

async function remove(a: Account) {
  if (!confirm(`确定删除账户「${a.name}」吗？`)) return;
  try {
    await api.deleteAccount(a.id);
    toast.success("已删除");
    await reloadMeta();
  } catch (e) {
    toast.error(String(e));
  }
}
</script>

<template>
  <div class="manage">
    <div class="add-row">
      <input v-model="newName" placeholder="新账户名称（如：招行卡）" @keyup.enter="add" />
      <button class="btn btn-primary" @click="add">添加</button>
    </div>

    <div class="card">
      <div v-for="a in store.accounts" :key="a.id" class="row" :class="{ off: a.status === 0 }">
        <template v-if="editingId === a.id">
          <input v-model="editingName" class="edit-input" @keyup.enter="saveEdit" />
          <button class="btn btn-primary" @click="saveEdit">保存</button>
          <button class="btn btn-text" @click="editingId = null">取消</button>
        </template>
        <template v-else>
          <span class="name">{{ a.name }}</span>
          <span v-if="a.status === 0" class="badge">已停用</span>
          <span class="spacer" />
          <button class="btn btn-text" @click="startEdit(a)">重命名</button>
          <button class="btn btn-text" @click="toggle(a)">{{ a.status === 1 ? "停用" : "启用" }}</button>
          <button class="btn btn-danger-text" @click="remove(a)">删除</button>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.manage {
  display: flex;
  flex-direction: column;
  gap: 12px;
  max-width: 720px;
}

.add-row {
  display: flex;
  gap: 8px;
}

.add-row input {
  flex: 1;
}

.row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 4px;
  border-bottom: 1px solid var(--line);
}

.row:last-child {
  border-bottom: none;
}

.row.off {
  opacity: 0.55;
}

.name {
  font-weight: 600;
}

.badge {
  padding: 2px 8px;
  border-radius: 10px;
  background: #f2f3f5;
  color: var(--text-3);
  font-size: 12px;
}

.spacer {
  flex: 1;
}

.edit-input {
  flex: 1;
}
</style>
