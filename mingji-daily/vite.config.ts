import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// Tauri 官方推荐配置：固定 1420 端口，忽略 src-tauri 的监听
export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
