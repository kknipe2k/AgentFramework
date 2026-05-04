import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// Tauri-recommended config: don't clear screen, use a fixed port,
// expose env vars as VITE_*. Per https://v2.tauri.app/start/frontend/.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: false,
  },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: {
    target: 'es2022',
    minify: 'esbuild',
    sourcemap: true,
    chunkSizeWarningLimit: 600,
  },
});
