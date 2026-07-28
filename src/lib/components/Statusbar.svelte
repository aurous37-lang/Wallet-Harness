<script lang="ts">
  import { api, isTauri, type AppInfo } from '$lib/ipc';
  import { createChainStatusStore } from '$lib/stores/chainStatus.svelte';

  let info = $state<AppInfo | null>(null);
  let clock = $state('');

  const chain = createChainStatusStore();

  $effect(() => {
    if (isTauri()) {
      api.meta.appInfo().then(i => { info = i; }).catch(() => {});
    }
    const tick = () => {
      const d = new Date();
      const hh = String(d.getUTCHours()).padStart(2, '0');
      const mm = String(d.getUTCMinutes()).padStart(2, '0');
      const ss = String(d.getUTCSeconds()).padStart(2, '0');
      clock = `${hh}:${mm}:${ss} UTC`;
    };
    tick();
    const id = setInterval(tick, 1000);
    return () => clearInterval(id);
  });

  const rpcLabel = $derived(
    chain.status.endpoint_label ?? (isTauri() ? 'idle' : 'browser'),
  );
  const blockDisplay = $derived(
    chain.status.block_number !== null
      ? `#${chain.status.block_number.toLocaleString('en-US')}`
      : '—',
  );
  const latencyDisplay = $derived(
    chain.status.latency_ms !== null ? `${chain.status.latency_ms}ms` : '—',
  );
</script>

<footer class="statusbar">
  <div class="seg">
    <span class="dot" class:online={chain.online} class:offline={!chain.online}></span>
    <span>rpc: {rpcLabel}</span>
  </div>
  <div class="seg">
    <span>chain: {chain.status.chain_id}</span>
  </div>
  <div class="seg">
    <span>block: <span class="mono">{blockDisplay}</span></span>
  </div>
  <div class="seg">
    <span>lat: <span class="mono">{latencyDisplay}</span></span>
  </div>
  <div class="seg">
    <span>watcher: off</span>
  </div>
  <div class="seg flex-grow">
    <span class="muted">no external telemetry · keys never leave your signer</span>
  </div>
  <div class="seg">
    <span class="mono">{clock}</span>
  </div>
  <div class="seg">
    {#if info}
      <span class="mono">{info.version} · {info.build_mode}</span>
    {:else}
      <span class="mono">—</span>
    {/if}
  </div>
</footer>

<style>
  .statusbar {
    display: flex;
    align-items: center;
    gap: var(--s-4);
    padding: 0 var(--s-3);
    background: var(--bg-1);
    border-top: 1px solid var(--border);
    height: var(--statusbar-h);
    font-family: var(--font-data);
    font-size: var(--fs-xs);
    color: var(--fg-2);
  }
  .seg {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }
  .seg.flex-grow { flex: 1; justify-content: flex-end; }
  .dot {
    width: 6px; height: 6px; border-radius: 50%;
    background: var(--fg-3);
    display: inline-block;
  }
  .dot.online { background: var(--gain); box-shadow: 0 0 6px var(--gain); }
  .dot.offline { background: var(--loss); }
  .muted { color: var(--fg-3); }
</style>
