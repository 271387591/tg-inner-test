import { defineConfig } from "vite";
import { resolve } from 'path'
import vue from "@vitejs/plugin-vue";

const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  resolve:{
    alias:{
      '@': resolve(__dirname, 'src'),
    },
    // extensions: ['.js', '.ts', '.vue']  // 确保自动解析 .vue
  },
  build: {
    // 设置devtool选项
    rollupOptions: {
      output: {
        // 设置inline-source-map
        devtool: 'inline-source-map',
      },
    },
  },
  plugins: [vue()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 4173,
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

      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
