<script lang="ts">
  import '../app.css';
  import { page } from '$app/stores';
  import { isTauri } from '$lib/ipc';
  import Topbar from '$lib/components/Topbar.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import Statusbar from '$lib/components/Statusbar.svelte';

  let { children } = $props();
</script>

<div class="shell">
  <Topbar />
  <div class="body">
    <Sidebar currentPath={$page.url.pathname} />
    <main class="content">
      {#if !isTauri()}
        <div class="dev-banner">
          ⚠ running in browser mode — Tauri IPC is unavailable; commands return stubs
        </div>
      {/if}
      {@render children()}
    </main>
  </div>
  <Statusbar />
</div>

<style>
  .shell {
    display: grid;
    grid-template-rows: var(--topbar-h) 1fr var(--statusbar-h);
    height: 100vh;
    width: 100vw;
  }
  .body {
    display: grid;
    grid-template-columns: var(--sidebar-w) 1fr;
    min-height: 0;
  }
  .content {
    overflow: auto;
    background: var(--bg-0);
    padding: var(--s-3);
    min-width: 0;
  }
  .dev-banner {
    background: #3d2a00;
    color: var(--warn);
    border: 1px solid var(--accent-dim);
    padding: var(--s-2) var(--s-3);
    font-size: var(--fs-xs);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    margin-bottom: var(--s-3);
  }
</style>
