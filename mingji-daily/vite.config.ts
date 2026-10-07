import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// Tauri 官方推荐配置：固定 1420 端口，忽略 src-tauri 的监听
// - 桌面调试（F5）：仅监听本机
// - 移动端调试（tauri android/ios dev）：Tauri 注入 TAURI_DEV_HOST（局域网 IP），
//   此时必须监听该地址，模拟器/真机才能加载前端资源，否则应用白屏
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
