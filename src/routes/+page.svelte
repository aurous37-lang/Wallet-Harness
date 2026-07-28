<script lang="ts">
  import { api, isTauri } from '$lib/ipc';
  import { createChainStatusStore } from '$lib/stores/chainStatus.svelte';

  let pingResult = $state<string>('—');
  let apiKeys = $state<string[]>([]);

  const chain = createChainStatusStore();

  $effect(() => {
    if (!isTauri()) return;
    api.meta.ping().then(r => { pingResult = r; }).catch(e => { pingResult = `error: ${e}`; });
    api.settings.listApiKeySlots().then(s => { apiKeys = s; }).catch(() => {});
  });

  const backendUp = $derived(pingResult === 'pong');
  const heroState = $derived(
    !isTauri() ? 'BROWSER' : backendUp && chain.online ? 'ONLINE' : backendUp ? 'DEGRADED' : 'OFFLINE',
  );
  const heroTone = $derived(
    heroState === 'ONLINE' ? 'ok' : heroState === 'DEGRADED' ? 'warn' : 'bad',
  );
  const blockDisplay = $derived(
    chain.status.block_number !== null
      ? `#${chain.status.block_number.toLocaleString('en-US')}`
      : '—',
  );
  const latencyDisplay = $derived(
    chain.status.latency_ms !== null ? `${chain.status.latency_ms}ms` : '—',
  );
  const endpointDisplay = $derived(chain.status.endpoint_label ?? '—');
</script>

<svelte:head><title>overview — wallet harness</title></svelte:head>

<div class="grid">
  <!-- hero strip -->
  <section class="hero panel">
    <div class="hero-head">
      <div>
        <div class="eyebrow">session ready</div>
        <h1>local exchange terminal</h1>
        <p class="tagline">
          your wallet, your rpc, your data. nothing leaves this device without your explicit approval.
        </p>
      </div>
      <div class="hero-state">
        <div class="big-num mono" class:ok={heroTone === 'ok'} class:warn={heroTone === 'warn'} class:bad={heroTone === 'bad'}>{heroState}</div>
        <div class="eyebrow">backend status</div>
        <div class="hero-metrics">
          <div class="metric">
            <div class="eyebrow">monad block</div>
            <div class="metric-num mono">{blockDisplay}</div>
          </div>
          <div class="metric">
            <div class="eyebrow">latency</div>
            <div class="metric-num mono">{latencyDisplay}</div>
          </div>
          <div class="metric">
            <div class="eyebrow">endpoint</div>
            <div class="metric-num mono endpoint">{endpointDisplay}</div>
          </div>
        </div>
      </div>
    </div>
  </section>

  <!-- quick stats row -->
  <section class="stat panel">
    <div class="panel-header"><span>portfolio value</span></div>
    <div class="panel-body">
      <div class="stat-num mono">$ —</div>
      <div class="stat-sub">connect a wallet to populate</div>
    </div>
  </section>

  <section class="stat panel">
    <div class="panel-header"><span>open orders</span></div>
    <div class="panel-body">
      <div class="stat-num mono">0</div>
      <div class="stat-sub">armed · triggered · awaiting sig</div>
    </div>
  </section>

  <section class="stat panel">
    <div class="panel-header"><span>24h realized p&amp;l</span></div>
    <div class="panel-body">
      <div class="stat-num mono">$ —</div>
      <div class="stat-sub">fifo · configurable jurisdiction rate</div>
    </div>
  </section>

  <section class="stat panel">
    <div class="panel-header"><span>api keys configured</span></div>
    <div class="panel-body">
      <div class="stat-num mono">{apiKeys.length}</div>
      <div class="stat-sub">{apiKeys.length ? apiKeys.join(' · ') : 'none — all calls will fail until configured'}</div>
    </div>
  </section>

  <!-- build order pane -->
  <section class="build panel">
    <div class="panel-header"><span>v1 build sequence</span></div>
    <div class="panel-body">
      <ol class="checklist">
        <li><span class="seq">01</span> scaffold &amp; signer abstraction <span class="state done">complete</span></li>
        <li><span class="seq">02</span> chain-rpc + monad read path <span class="state todo">todo</span></li>
        <li><span class="seq">03</span> cost-basis port (fifo, configurable rate) <span class="state todo">todo</span></li>
        <li><span class="seq">04</span> charts + pricing <span class="state todo">todo</span></li>
        <li><span class="seq">05</span> manual swaps via walletconnect <span class="state todo">todo</span></li>
        <li><span class="seq">06</span> armed-order engine (no autonomous) <span class="state todo">todo</span></li>
        <li><span class="seq">07</span> hardware signer (manual only) <span class="state todo">todo</span></li>
        <li><span class="seq">08</span> multi-chain portfolio view <span class="state todo">todo</span></li>
        <li><span class="seq">09</span> windows + macos packaging <span class="state todo">todo</span></li>
        <li><span class="seq">10</span> android companion <span class="state todo">todo</span></li>
      </ol>
      <div class="gate">
        <span class="gate-label">v2 gate:</span>
        <span>autonomous hot-keystore signing unlocks when live strategy win rate ≥ 73%</span>
      </div>
    </div>
  </section>

  <section class="principles panel">
    <div class="panel-header"><span>principles</span></div>
    <div class="panel-body">
      <ul>
        <li><span class="k">01</span> the app holds no keys in v1.</li>
        <li><span class="k">02</span> no telemetry, no analytics, no remote logs.</li>
        <li><span class="k">03</span> every external call is opt-in and user-configured.</li>
        <li><span class="k">04</span> cost basis is deterministic &amp; reproducible from chain history.</li>
      </ul>
    </div>
  </section>
</div>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: var(--s-3);
  }
  .hero { grid-column: span 4; }
  .stat { grid-column: span 1; }
  .build { grid-column: span 3; }
  .principles { grid-column: span 1; }

  .hero-head {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: var(--s-6);
    padding: var(--s-6);
    align-items: start;
  }
  .eyebrow {
    font-size: var(--fs-xs);
    text-transform: uppercase;
    letter-spacing: 0.18em;
    color: var(--fg-2);
    margin-bottom: var(--s-2);
  }
  .hero h1 {
    font-family: var(--font-display);
    font-weight: 700;
    font-size: var(--fs-xxl);
    line-height: 1.1;
    letter-spacing: -0.01em;
    color: var(--fg-0);
    text-transform: lowercase;
    margin-bottom: var(--s-3);
  }
  .tagline {
    color: var(--fg-1);
    font-size: var(--fs-md);
    max-width: 62ch;
  }
  .hero-state {
    text-align: right;
  }
  .big-num {
    font-size: var(--fs-xxl);
    color: var(--accent);
    letter-spacing: 0.04em;
    line-height: 1;
    margin-bottom: var(--s-2);
  }
  .big-num.ok   { color: var(--gain); }
  .big-num.warn { color: var(--warn); }
  .big-num.bad  { color: var(--loss); }
  .hero-metrics {
    margin-top: var(--s-4);
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
    align-items: flex-end;
  }
  .metric-num {
    font-size: var(--fs-md);
    color: var(--fg-0);
    letter-spacing: 0.02em;
  }
  .metric-num.endpoint {
    color: var(--fg-1);
    font-size: var(--fs-sm);
    max-width: 28ch;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .stat-num {
    font-size: var(--fs-xl);
    color: var(--fg-0);
    letter-spacing: 0.02em;
    margin-bottom: var(--s-1);
  }
  .stat-sub {
    font-size: var(--fs-xs);
    color: var(--fg-2);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .checklist {
    list-style: none;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 2px;
  }
  .checklist li {
    display: grid;
    grid-template-columns: 32px 1fr auto;
    align-items: center;
    gap: var(--s-2);
    padding: 6px var(--s-2);
    font-size: var(--fs-sm);
    border-bottom: 1px solid var(--border);
  }
  .seq {
    font-family: var(--font-data);
    color: var(--fg-3);
    font-size: var(--fs-xs);
  }
  .state {
    font-size: var(--fs-xs);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    padding: 1px 6px;
    border: 1px solid currentColor;
  }
  .state.done { color: var(--gain); }
  .state.todo { color: var(--fg-3); }
  .gate {
    margin-top: var(--s-4);
    padding: var(--s-3);
    border: 1px solid var(--accent-dim);
    background: rgba(184, 120, 0, 0.08);
    font-size: var(--fs-sm);
    display: flex;
    gap: var(--s-2);
    color: var(--fg-1);
  }
  .gate-label {
    color: var(--accent);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    font-size: var(--fs-xs);
  }

  .principles ul {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: var(--s-3);
  }
  .principles li {
    font-size: var(--fs-sm);
    color: var(--fg-1);
    display: grid;
    grid-template-columns: 28px 1fr;
    gap: var(--s-2);
    align-items: start;
  }
  .k {
    font-family: var(--font-data);
    color: var(--accent);
    font-size: var(--fs-xs);
  }
</style>
