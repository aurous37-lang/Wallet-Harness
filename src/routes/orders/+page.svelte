<script lang="ts">
  import { type Order } from '$lib/ipc';

  let orders = $state<Order[]>([]);

  const cols = [
    { key: 'Armed',            label: 'armed',              hint: 'trigger condition not yet met' },
    { key: 'Triggered',        label: 'triggered',          hint: 'condition hit · preparing swap' },
    { key: 'AwaitingSignature',label: 'awaiting signature', hint: 'ready for you to approve on your signer' },
    { key: 'Submitted',        label: 'submitted',          hint: 'on chain · awaiting receipt' },
  ] as const;

  function byStatus(status: string): Order[] {
    return orders.filter(o => o.status === status);
  }
</script>

<svelte:head><title>orders — wallet harness</title></svelte:head>

<section class="panel">
  <div class="panel-header">
    <span>armed-order pipeline</span>
    <span class="small">{orders.length} total · v1 model: triggers notify, you sign</span>
  </div>
  <div class="panel-body">
    <div class="kanban">
      {#each cols as col}
        <div class="col">
          <div class="col-head">
            <span class="col-label pill {col.key.toLowerCase()}">{col.label}</span>
            <span class="col-count mono">{byStatus(col.key).length}</span>
          </div>
          <div class="col-hint">{col.hint}</div>
          <div class="col-body">
            {#each byStatus(col.key) as o}
              <div class="card">
                <div class="row"><span>{o.pair}</span><span class="pill {o.status.toLowerCase()}">{o.kind}</span></div>
                <div class="row mono"><span>trigger</span><span>${o.trigger_price_usd}</span></div>
                <div class="row mono"><span>size</span><span>{o.size_token}</span></div>
              </div>
            {:else}
              <div class="empty">—</div>
            {/each}
          </div>
        </div>
      {/each}
    </div>

    <div class="footnote">
      autonomous execution is <strong>disabled in v1</strong>. orders that trigger surface in the
      <em>awaiting signature</em> column and wait for your explicit approval through the connected signer.
      autonomous signing unlocks in v2 once a strategy demonstrates ≥ 73% live win rate.
    </div>
  </div>
</section>

<style>
  .small { font-size: var(--fs-xs); color: var(--fg-2); }
  .kanban {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: var(--s-3);
    margin-bottom: var(--s-4);
  }
  .col {
    background: var(--bg-2);
    border: 1px solid var(--border);
    min-height: 300px;
    display: flex; flex-direction: column;
  }
  .col-head {
    display: flex; justify-content: space-between; align-items: center;
    padding: var(--s-2) var(--s-3);
    border-bottom: 1px solid var(--border);
  }
  .col-count { font-size: var(--fs-sm); color: var(--fg-2); }
  .col-hint {
    font-size: var(--fs-xs);
    color: var(--fg-3);
    padding: var(--s-2) var(--s-3);
    border-bottom: 1px dashed var(--border);
  }
  .col-body {
    padding: var(--s-2);
    flex: 1;
    display: flex; flex-direction: column;
    gap: var(--s-2);
  }
  .card {
    background: var(--bg-1);
    border: 1px solid var(--border);
    padding: var(--s-2);
    font-size: var(--fs-sm);
  }
  .card .row {
    display: flex; justify-content: space-between;
    padding: 1px 0;
  }
  .empty { color: var(--fg-3); text-align: center; padding: var(--s-4); font-size: var(--fs-sm); }
  .footnote {
    font-size: var(--fs-sm);
    color: var(--fg-2);
    background: var(--bg-2);
    border-left: 2px solid var(--accent-dim);
    padding: var(--s-3);
    line-height: 1.6;
  }
  .footnote strong { color: var(--accent); }
  .footnote em { color: var(--fg-0); font-style: normal; text-decoration: underline; text-decoration-color: var(--border-hi); text-underline-offset: 3px; }
</style>
