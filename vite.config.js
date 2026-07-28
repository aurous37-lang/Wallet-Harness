import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

// Tauri defaults tuned for fast HMR and correct dev-server binding.
// Tauri expects a fixed port; avoid auto-increment so the backend webview
// can always reach the frontend at `devUrl` from tauri.conf.json.
export default defineConfig(async () => ({
  plugins: [sveltekit()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    host: '0.0.0.0',
    watch: {
      // Ignore Rust target dir so Vite doesn't thrash when Cargo rebuilds.
      ignored: ['**/src-tauri/**', '**/target/**']
    }
  },
  // Environment variables prefixed TAURI_ are exposed at build time for
  // platform gating in the frontend.
  envPrefix: ['VITE_', 'TAURI_ENV_']
}));
