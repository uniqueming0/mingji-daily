<script setup lang="ts">
import { ref } from "vue";
import { api } from "../api";
import { toast } from "../toast";
import { reloadMeta, store } from "../store";
import type { Category } from "../types";

const newTopName = ref("");
const newChildParent = ref<number | null>(null);
const newChildName = ref("");
const editingId = ref<number | null>(null);
const editingName = ref("");

const tops = () => store.categories.filter((c) => c.parent_id === null);
const children = (pid: number) => store.categories.filter((c) => c.parent_id === pid);

async function addTop() {
  const name = newTopName.value.trim();
  if (!name) return;
  try {
    await api.createCategory(name, null);
    toast.success("已添加");
    newTopName.value = "";
    await reloadMeta();
  } catch (e) {
    toast.error(String(e));
  }
}

async function addChild(pid: number) {
  const name = newChildName.value.trim();
  if (!name) return;
  try {
    await api.createCategory(name, pid);
    toast.success("已添加");
    newChildParent.value = null;
    newChildName.value = "";
    await reloadMeta();
  } catch (e) {
    toast.error(String(e));
  }
}

function startEdit(c: Category) {
  editingId.value = c.id;
  editingName.value = c.name;
}

async function saveEdit() {
  if (editingId.value == null) return;
  const name = editingName.value.trim();
  if (!name) return;
  const cur = store.categories.find((c) => c.id === editingId.value);
  if (!cur) return;
  try {
    await api.updateCategory(editingId.value, name, cur.status);
    toast.success("已保存");
    editingId.value = null;
    await reloadMeta();
  } catch (e) {
    toast.error(String(e));
  }
}

async function toggle(c: Category) {
  try {
    await api.updateCategory(c.id, c.name, c.status === 1 ? 0 : 1);
    toast.success(c.status === 1 ? "已停用" : "已启用");
    await reloadMeta();
  } catch (e) {
    toast.error(String(e));
  }
}

async function remove(c: Category) {
  if (!confirm(`确定删除分类「${c.name}」吗？`)) return;
  try {
    await api.deleteCategory(c.id);
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
      <input v-model="newTopName" placeholder="新一级分类名称" @keyup.enter="addTop" />
      <button class="btn btn-primary" @click="addTop">添加</button>
    </div>

    <div v-for="t in tops()" :key="t.id" class="top card" :class="{ off: t.status === 0 }">
      <div class="top-row">
        <template v-if="editingId === t.id">
          <input v-model="editingName" class="edit-input" @keyup.enter="saveEdit" />
          <button class="btn btn-primary" @click="saveEdit">保存</button>
          <button class="btn btn-text" @click="editingId = null">取消</button>
        </template>
        <template v-else>
          <span class="name">{{ t.name }}</span>
          <span v-if="t.status === 0" class="badge">已停用</span>
          <span class="spacer" />
          <button class="btn btn-text" @click="newChildParent = t.id">＋子分类</button>
          <button class="btn btn-text" @click="startEdit(t)">重命名</button>
          <button class="btn btn-text" @click="toggle(t)">{{ t.status === 1 ? "停用" : "启用" }}</button>
          <button class="btn btn-danger-text" @click="remove(t)">删除</button>
        </template>
      </div>

      <div v-if="newChildParent === t.id" class="add-row child-add">
        <input v-model="newChildName" placeholder="子分类名称" @keyup.enter="addChild(t.id)" />
        <button class="btn btn-primary" @click="addChild(t.id)">添加</button>
        <button class="btn btn-text" @click="newChildParent = null">取消</button>
      </div>

      <div v-if="children(t.id).length" class="children">
        <div v-for="c in children(t.id)" :key="c.id" class="child-row" :class="{ off: c.status === 0 }">
          <template v-if="editingId === c.id">
            <input v-model="editingName" class="edit-input" @keyup.enter="saveEdit" />
            <button class="btn btn-primary" @click="saveEdit">保存</button>
            <button class="btn btn-text" @click="editingId = null">取消</button>
          </template>
          <template v-else>
            <span class="name">{{ c.name }}</span>
            <span v-if="c.status === 0" class="badge">已停用</span>
            <span class="spacer" />
            <button class="btn btn-text" @click="startEdit(c)">重命名</button>
            <button class="btn btn-text" @click="toggle(c)">{{ c.status === 1 ? "停用" : "启用" }}</button>
            <button class="btn btn-danger-text" @click="remove(c)">删除</button>
          </template>
        </div>
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

.child-add {
  margin-top: 10px;
}

.top.off,
.child-row.off {
  opacity: 0.55;
}

.top-row,
.child-row {
  display: flex;
  align-items: center;
  gap: 8px;
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

.children {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-top: 10px;
  padding-left: 16px;
  border-left: 2px solid var(--line);
}

.child-row {
  padding: 6px 0;
  font-size: 13px;
  color: var(--text-2);
}
</style>
