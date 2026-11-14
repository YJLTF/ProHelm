import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// Tauri 前端构建配置：固定端口 1420，与 src-tauri/tauri.conf.json 的 devUrl 对齐
export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  resolve: {
    alias: {
      // frappe-gantt 的 exports 未暴露子路径 CSS，用别名指向实际文件
      "frappe-gantt/dist/frappe-gantt.css": fileURLToPath(
        new URL("./node_modules/frappe-gantt/dist/frappe-gantt.css", import.meta.url),
      ),
    },
  },
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    target: "es2022",
  },
});
