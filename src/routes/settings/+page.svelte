<script lang="ts">
  import { api, isTauri, type SignerOption } from '$lib/ipc';

  let tab = $state<'wallet' | 'rpc' | 'keys' | 'tax'>('wallet');
  let signerOptions = $state<SignerOption[]>([]);
  let apiKeySlots = $state<string[]>([]);

  $effect(() => {
    if (!isTauri()) return;
    api.wallet.listSignerTypes().then(s => { signerOptions = s; }).catch(() => {});
    api.settings.listApiKeySlots().then(s => { apiKeySlots = s; }).catch(() => {});
  });
</script>

<svelte:head><title>settings — wallet harness</title></svelte:head>

<section class="panel">
  <div class="panel-header"><span>settings</span></div>
  <div class="panel-body flush">
    <nav class="tabs">
      <button class:active={tab === 'wallet'} onclick={() => tab = 'wallet'}>wallet / signer</button>
      <button class:active={tab === 'rpc'} onclick={() => tab = 'rpc'}>rpc endpoints</button>
      <button class:active={tab === 'keys'} onclick={() => tab = 'keys'}>api keys</button>
      <button class:active={tab === 'tax'} onclick={() => tab = 'tax'}>tax config</button>
    </nav>

    <div class="tab-body">
      {#if tab === 'wallet'}
        <p class="intro">
          every signing transport is non-custodial. the app never touches your keys.
          pick one that fits the platform you're on and the security posture you want.
        </p>
        <div class="signer-grid">
          {#each signerOptions as s}
            <div class="signer-card" class:unsupported={!s.platform_supported}>
              <div class="sc-head">
                <span class="sc-name">{s.label}</span>
                {#if !s.platform_supported}
                  <span class="pill cancelled">unsupported on this platform</span>
                {/if}
              </div>
              <div class="sc-meta">
                <div class="row"><span>transport</span><span class="mono">{s.capabilities.transport}</span></div>
                <div class="row"><span>autonomous</span><span class="mono">{s.capabilities.autonomous ? 'yes' : 'no'}</span></div>
                <div class="row"><span>eip-1559</span><span class="mono">{s.capabilities.supports_eip1559 ? 'yes' : 'no'}</span></div>
                <div class="row"><span>eip-712</span><span class="mono">{s.capabilities.supports_eip712 ? 'yes' : 'no'}</span></div>
              </div>
              <button class="btn-primary full" disabled={!s.platform_supported}>
                connect via {s.capabilities.transport.toLowerCase()}
              </button>
            </div>
          {:else}
            <div class="scaffold">
              <h2>no signers available</h2>
              <p>signer listings are provided by the backend via <code>list_signer_types</code>. in dev-browser mode this returns empty; run via <code>pnpm tauri dev</code>.</p>
            </div>
          {/each}
        </div>

      {:else if tab === 'rpc'}
        <p class="intro">rpc endpoints per chain. bring your own — alchemy, infura, quicknode, or your own node. the app tries them in priority order with automatic failover.</p>
        <div class="scaffold">
          <h2>add rpc endpoint</h2>
          <p>form with chain / label / url / priority. url may embed <code>{'{ALCHEMY_KEY}'}</code> placeholders that resolve from the keychain at call time.</p>
        </div>

      {:else if tab === 'keys'}
        <p class="intro">
          api keys live in the os keychain — secret service on linux, credential manager on windows,
          keychain on macos. the app database never stores them. only slot names appear in the db.
        </p>
        <div class="keys-list">
          {#each ['Alchemy','Infura','QuickNode','CoinGecko','DexScreener','OneInch','Paraswap'] as slot}
            <div class="key-row">
              <span class="key-name">{slot}</span>
              <span class="pill {apiKeySlots.includes(slot) ? 'confirmed' : 'cancelled'}">
                {apiKeySlots.includes(slot) ? 'configured' : 'empty'}
              </span>
              <button class="btn-ghost">set</button>
            </div>
          {/each}
        </div>

      {:else if tab === 'tax'}
        <p class="intro">cost-basis ledger uses fifo. federal rates are computed at report time. the jurisdiction rate is configurable and defaults to 4.4%.</p>
        <div class="form-row">
          <label for="jur">jurisdiction</label>
          <input id="jur" type="text" value="" placeholder="optional jurisdiction code" />
        </div>
        <div class="form-row">
          <label for="rate">jurisdiction rate (decimal)</label>
          <input id="rate" type="text" value="0.044" />
        </div>
        <button class="btn">save tax config</button>
      {/if}
    </div>
  </div>
</section>

<style>
  .tabs {
    display: flex;
    border-bottom: 1px solid var(--border);
    background: var(--bg-1);
  }
  .tabs button {
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    padding: 10px var(--s-4);
    color: var(--fg-1);
    font-size: var(--fs-sm);
    text-transform: lowercase;
    letter-spacing: 0.04em;
  }
  .tabs button:hover { color: var(--fg-0); background: var(--bg-2); }
  .tabs button.active {
    color: var(--accent);
    border-bottom-color: var(--accent);
  }

  .tab-body { padding: var(--s-4); }
  .intro {
    color: var(--fg-1);
    font-size: var(--fs-sm);
    max-width: 80ch;
    margin-bottom: var(--s-4);
  }

  .signer-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: var(--s-3);
  }
  .signer-card {
    background: var(--bg-2);
    border: 1px solid var(--border);
    padding: var(--s-3);
    display: flex; flex-direction: column; gap: var(--s-3);
  }
  .signer-card.unsupported { opacity: 0.5; }
  .sc-head {
    display: flex; justify-content: space-between; align-items: start;
    gap: var(--s-2);
  }
  .sc-name {
    font-size: var(--fs-md);
    color: var(--fg-0);
    font-weight: 500;
  }
  .sc-meta { display: flex; flex-direction: column; gap: 2px; }
  .sc-meta .row {
    display: flex; justify-content: space-between;
    font-size: var(--fs-xs);
    color: var(--fg-2);
  }
  .sc-meta .row span:last-child { color: var(--fg-0); }
  .btn-primary.full { width: 100%; }

  .keys-list { display: flex; flex-direction: column; gap: 2px; }
  .key-row {
    display: grid;
    grid-template-columns: 1fr auto auto;
    align-items: center;
    gap: var(--s-3);
    padding: var(--s-2) var(--s-3);
    background: var(--bg-2);
    border: 1px solid var(--border);
  }
  .key-name { font-family: var(--font-data); font-size: var(--fs-sm); }

  .form-row { margin-bottom: var(--s-3); max-width: 400px; }
</style>
