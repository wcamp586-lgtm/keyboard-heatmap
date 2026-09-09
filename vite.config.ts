import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'
import { readFileSync } from 'node:fs'

const readProjectFile = (path: string) => readFileSync(new URL(path, import.meta.url), 'utf8')
const version: string = JSON.parse(readProjectFile('./package.json')).version
const nativeVersion: string = JSON.parse(readProjectFile('./src-tauri/tauri.conf.json')).version
const cargoVersion = readProjectFile('./src-tauri/Cargo.toml').match(/^version = "([^"]+)"/m)?.[1]
if (version !== nativeVersion || version !== cargoVersion) {
  throw new Error('前端、Tauri 与 Cargo 版本不一致，停止构建。')
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  define: { __APP_VERSION__: JSON.stringify(version) },
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
})
