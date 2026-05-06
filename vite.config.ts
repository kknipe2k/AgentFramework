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
    // Bind explicitly to the IPv4 loopback. Vite 7 defaults to `'localhost'`
    // which Node 17+ resolves to `::1` first (IPv6) — tools that connect via
    // IPv4 (Playwright's bundled Chromium, some curl builds) then fail with
    // ECONNREFUSED / connection-reset. Pinning to 127.0.0.1 sidesteps the
    // dns-resolution-order skew without exposing the server on a wildcard
    // interface.
    host: '127.0.0.1',
  },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  // Pre-bundle frontend deps at server start so the first page load doesn't
  // wait on the optimizer mid-request. Without this, Playwright's `page.goto`
  // can exceed its default 30s when `node_modules/.vite/deps` is cold (Vite 7
  // worsens this slightly because of Rolldown-scout warm-up behavior).
  optimizeDeps: {
    include: [
      '@tauri-apps/api/core',
      '@tauri-apps/api/event',
      'react',
      'react-dom',
      'react-dom/client',
      'react/jsx-dev-runtime',
    ],
  },
  build: {
    target: 'es2022',
    minify: 'esbuild',
    sourcemap: true,
    chunkSizeWarningLimit: 600,
  },
});
