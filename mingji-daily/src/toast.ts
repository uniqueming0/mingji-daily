import { reactive } from "vue";

interface ToastItem {
  id: number;
  type: "success" | "error" | "info";
  text: string;
}

export const toasts = reactive<ToastItem[]>([]);
let seq = 0;

function push(type: ToastItem["type"], text: string) {
  const id = ++seq;
  toasts.push({ id, type, text });
  setTimeout(() => {
    const i = toasts.findIndex((t) => t.id === id);
    if (i >= 0) toasts.splice(i, 1);
  }, 2600);
}

export const toast = {
  success: (text: string) => push("success", text),
  error: (text: string) => push("error", text),
  info: (text: string) => push("info", text),
};
