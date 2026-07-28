<script lang="ts">
  import { api, isTauri, type ConnectedAccount } from '$lib/ipc';

  let account = $state<ConnectedAccount | null>(null);
  let activeChain = $state('Monad');

  const chains = ['Monad', 'Ethereum', 'Arbitrum', 'Base', 'BNB'];

  $effect(() => {
    if (!isTauri()) return;
    api.wallet.activeAccount().then(a => { account = a; }).catch(() => {});
  });

  function shortAddr(a: string): string {
    return a.length > 10 ? `${a.slice(0, 6)}…${a.slice(-4)}` : a;
  }
</script>

<header class="topbar">
  <div class="brand">
    <span class="mark">◆</span>
    <span class="name">WALLET HARNESS</span>
    <span class="version">v0.1.0</span>
  </div>

  <div class="middle">
    <div class="chain-select">
      <span class="label">chain</span>
      <select bind:value={activeChain}>
        {#each chains as c}
          <option value={c}>{c}</option>
        {/each}
      </select>
    </div>
  </div>

  <div class="right">
    {#if account}
      <div class="account">
        <span class="pill awaiting">{account.signer_kind}</span>
        <span class="addr mono">{shortAddr(account.address)}</span>
      </div>
    {:else}
      <a class="btn" href="/settings?tab=wallet">connect wallet</a>
    {/if}
  </div>
</header>

<style>
  .topbar {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: var(--s-4);
    padding: 0 var(--s-4);
    background: var(--bg-1);
    border-bottom: 1px solid var(--border);
    height: var(--topbar-h);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }
  .mark {
    color: var(--accent);
    font-size: var(--fs-lg);
    line-height: 1;
  }
  .name {
    font-family: var(--font-display);
    font-weight: 700;
    letter-spacing: 0.14em;
    font-size: var(--fs-sm);
  }
  .version {
    color: var(--fg-3);
    font-family: var(--font-data);
    font-size: var(--fs-xs);
  }
  .middle {
    display: flex;
    justify-content: center;
  }
  .chain-select {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }
  .chain-select .label {
    font-size: var(--fs-xs);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--fg-2);
  }
  .chain-select select {
    width: auto;
    padding: 3px 8px;
    font-size: var(--fs-sm);
  }
  .right {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }
  .account {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }
  .addr {
    color: var(--fg-0);
    font-size: var(--fs-sm);
  }
</style>
