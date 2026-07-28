<script lang="ts">
  let side = $state<'buy' | 'sell'>('buy');
  let orderType = $state<'market' | 'limit' | 'stop_loss'>('market');
  let amount = $state('');
  let triggerPrice = $state('');
  let slippageBps = $state('50');
  let preferProtocolOrder = $state(true);
  let privateMempool = $state(false);

  const tokenIn = $state('MON');
  const tokenOut = $state('USDC');
</script>

<svelte:head><title>trade — wallet harness</title></svelte:head>

<div class="grid">
  <!-- chart zone (top-left, wide) -->
  <section class="chart panel">
    <div class="panel-header">
      <span>{tokenIn} / {tokenOut}</span>
      <div class="tf-row">
        <button class="btn-ghost tf">1m</button>
        <button class="btn-ghost tf">5m</button>
        <button class="btn-ghost tf active">15m</button>
        <button class="btn-ghost tf">1h</button>
        <button class="btn-ghost tf">4h</button>
        <button class="btn-ghost tf">1d</button>
      </div>
    </div>
    <div class="panel-body flush chart-area">
      <div class="scaffold">
        <h2>chart renders here</h2>
        <p>tradingview lightweight-charts wired to <code>api.market.candles(chainId, pool, timeframeSeconds, limit)</code>. indexer-derived by default; dexscreener/coingecko fallback opt-in via <code>settings → data sources</code>.</p>
      </div>
    </div>
  </section>

  <!-- order form (right column) -->
  <section class="order panel">
    <div class="panel-header"><span>order</span></div>
    <div class="panel-body">
      <div class="side-toggle">
        <button class:active={side === 'buy'} class:buy={side === 'buy'}
                onclick={() => side = 'buy'}>buy</button>
        <button class:active={side === 'sell'} class:sell={side === 'sell'}
                onclick={() => side = 'sell'}>sell</button>
      </div>

      <div class="type-toggle">
        <button class:active={orderType === 'market'} onclick={() => orderType = 'market'}>market</button>
        <button class:active={orderType === 'limit'} onclick={() => orderType = 'limit'}>limit</button>
        <button class:active={orderType === 'stop_loss'} onclick={() => orderType = 'stop_loss'}>stop-loss</button>
      </div>

      <div class="form-row">
        <label for="amount">amount ({tokenIn})</label>
        <input id="amount" type="text" bind:value={amount} placeholder="0.00" />
      </div>

      {#if orderType !== 'market'}
        <div class="form-row">
          <label for="trigger">trigger price (usd)</label>
          <input id="trigger" type="text" bind:value={triggerPrice} placeholder="0.0000" />
        </div>
      {/if}

      <div class="form-row">
        <label for="slippage">slippage (bps)</label>
        <input id="slippage" type="text" bind:value={slippageBps} />
      </div>

      {#if orderType === 'limit'}
        <div class="form-row check">
          <input id="pref" type="checkbox" bind:checked={preferProtocolOrder} />
          <label for="pref">prefer on-chain limit protocol (1inch / cowswap)</label>
        </div>
      {/if}

      {#if orderType === 'market'}
        <div class="form-row check">
          <input id="priv" type="checkbox" bind:checked={privateMempool} />
          <label for="priv">private mempool (flashbots protect)</label>
        </div>
      {/if}

      <div class="quote-preview">
        <div class="row"><span>est. output</span><span class="mono">—</span></div>
        <div class="row"><span>price impact</span><span class="mono">—</span></div>
        <div class="row"><span>router</span><span class="mono">—</span></div>
        <div class="row"><span>est. gas</span><span class="mono">—</span></div>
      </div>

      <button class="btn-primary full">
        {#if orderType === 'market'}review &amp; sign{/if}
        {#if orderType === 'limit'}arm limit{/if}
        {#if orderType === 'stop_loss'}arm stop-loss{/if}
      </button>

      <div class="signer-note">
        <span class="pill awaiting">signer: not connected</span>
        <span class="hint">connect a signer in settings before placing orders.</span>
      </div>
    </div>
  </section>

  <!-- depth / book / recent fills zone -->
  <section class="depth panel">
    <div class="panel-header"><span>recent swaps</span></div>
    <div class="panel-body flush">
      <div class="scaffold">
        <h2>pool swap tape</h2>
        <p>streams swap events from the indexer for the active pair. color-coded buys/sells with per-swap size, price, and tx hash.</p>
      </div>
    </div>
  </section>
</div>

<style>
  .grid {
    display: grid;
    grid-template-columns: 1fr 320px;
    grid-template-rows: 1fr auto;
    gap: var(--s-3);
    min-height: calc(100vh - var(--topbar-h) - var(--statusbar-h) - var(--s-6));
  }
  .chart { grid-column: 1; grid-row: 1; min-height: 420px; }
  .order { grid-column: 2; grid-row: 1 / span 2; }
  .depth { grid-column: 1; grid-row: 2; min-height: 200px; }

  .chart-area { height: 100%; min-height: 380px; display: flex; align-items: stretch; }
  .chart-area .scaffold { flex: 1; display: flex; flex-direction: column; justify-content: center; }

  .tf-row { display: flex; gap: 2px; }
  .tf {
    padding: 2px 8px;
    font-size: var(--fs-xs);
    color: var(--fg-2);
    border: 1px solid transparent;
  }
  .tf:hover { color: var(--fg-0); background: var(--bg-2); }
  .tf.active { color: var(--accent); border-color: var(--accent-dim); }

  .side-toggle {
    display: grid; grid-template-columns: 1fr 1fr;
    gap: 2px;
    margin-bottom: var(--s-3);
  }
  .side-toggle button {
    padding: 8px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    font-size: var(--fs-xs);
  }
  .side-toggle .buy.active { background: var(--gain); color: var(--bg-0); border-color: var(--gain); }
  .side-toggle .sell.active { background: var(--loss); color: var(--bg-0); border-color: var(--loss); }

  .type-toggle {
    display: grid; grid-template-columns: repeat(3, 1fr);
    gap: 2px;
    margin-bottom: var(--s-3);
  }
  .type-toggle button {
    font-size: var(--fs-xs);
    padding: 6px;
    color: var(--fg-1);
  }
  .type-toggle button.active {
    color: var(--accent);
    border-color: var(--accent);
  }

  .form-row { margin-bottom: var(--s-3); }
  .form-row.check {
    display: flex; align-items: center; gap: var(--s-2);
  }
  .form-row.check input { width: auto; }
  .form-row.check label { margin-bottom: 0; text-transform: none; letter-spacing: 0; font-size: var(--fs-sm); color: var(--fg-1); }

  .quote-preview {
    background: var(--bg-2);
    border: 1px solid var(--border);
    padding: var(--s-2) var(--s-3);
    margin-bottom: var(--s-3);
  }
  .quote-preview .row {
    display: flex; justify-content: space-between;
    font-size: var(--fs-sm);
    color: var(--fg-1);
    padding: 2px 0;
  }
  .quote-preview .row span:last-child { color: var(--fg-0); }

  .btn-primary.full { width: 100%; padding: 10px; }

  .signer-note {
    margin-top: var(--s-3);
    font-size: var(--fs-xs);
    color: var(--fg-2);
    display: flex;
    flex-direction: column;
    gap: var(--s-1);
  }
  .hint { color: var(--fg-3); }
</style>
