<script lang="ts">
  import { api, isTauri, type Position, type TxRecord } from '$lib/ipc';

  let positions = $state<Position[]>([]);
  let history = $state<TxRecord[]>([]);
  let address = $state('');

  $effect(() => {
    if (!isTauri() || !address) return;
    api.portfolio.positions(address).then(p => { positions = p; }).catch(() => {});
    api.portfolio.history(address, undefined, 25).then(h => { history = h; }).catch(() => {});
  });
</script>

<svelte:head><title>portfolio — wallet harness</title></svelte:head>

<div class="cols">
  <section class="panel">
    <div class="panel-header">
      <span>positions · cost basis (fifo)</span>
      <span class="mono small">{positions.length} open</span>
    </div>
    <div class="panel-body flush">
      {#if positions.length === 0}
        <div class="scaffold">
          <h2>no positions indexed yet</h2>
          <p>connect a wallet in <code>settings → wallet</code>, configure an rpc endpoint for your active chain, then return here. positions populate from swap + transfer history.</p>
        </div>
      {:else}
        <table class="data">
          <thead>
            <tr>
              <th>token</th>
              <th class="num">qty</th>
              <th class="num">avg cost</th>
              <th class="num">mark</th>
              <th class="num">unrealized</th>
              <th class="num">realized</th>
              <th class="num">lots</th>
            </tr>
          </thead>
          <tbody>
            {#each positions as p}
              <tr>
                <td>{p.symbol}</td>
                <td class="num">{p.quantity.toFixed(6)}</td>
                <td class="num">${p.avg_cost_usd.toFixed(4)}</td>
                <td class="num">${p.market_price_usd.toFixed(4)}</td>
                <td class="num" class:gain={p.unrealized_pnl_usd > 0} class:loss={p.unrealized_pnl_usd < 0}>
                  ${p.unrealized_pnl_usd.toFixed(2)}
                </td>
                <td class="num" class:gain={p.realized_pnl_usd > 0} class:loss={p.realized_pnl_usd < 0}>
                  ${p.realized_pnl_usd.toFixed(2)}
                </td>
                <td class="num">{p.lot_count}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </div>
  </section>

  <section class="panel">
    <div class="panel-header">
      <span>transaction history</span>
      <span class="mono small">{history.length} records</span>
    </div>
    <div class="panel-body flush">
      {#if history.length === 0}
        <div class="scaffold">
          <h2>no transactions indexed</h2>
          <p>the local indexer reads erc-20 transfer + dex swap events directly from your rpc endpoint. history is reproducible from chain state alone.</p>
        </div>
      {:else}
        <table class="data">
          <thead>
            <tr>
              <th>time</th>
              <th>kind</th>
              <th>token</th>
              <th class="num">amount</th>
              <th>counterparty</th>
              <th>tx</th>
            </tr>
          </thead>
          <tbody>
            {#each history as tx}
              <tr>
                <td class="mono small">{new Date(tx.timestamp * 1000).toISOString().slice(0, 19).replace('T', ' ')}</td>
                <td>{tx.kind}</td>
                <td>{tx.token_symbol ?? '—'}</td>
                <td class="num">{tx.amount_ui?.toFixed(6) ?? '—'}</td>
                <td class="mono small">{tx.counterparty ?? '—'}</td>
                <td class="mono small">{tx.tx_hash.slice(0, 10)}…</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </div>
  </section>
</div>

<style>
  .cols {
    display: flex;
    flex-direction: column;
    gap: var(--s-3);
  }
  .small { font-size: var(--fs-xs); color: var(--fg-2); }
</style>
