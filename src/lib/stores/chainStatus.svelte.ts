import { api, isTauri, type ChainStatus } from '$lib/ipc';

export const MONAD_CHAIN_ID = 10143;
const POLL_MS = 4000;

function emptyStatus(chainId: number): ChainStatus {
  return {
    chain_id: chainId,
    block_number: null,
    latency_ms: null,
    endpoint_label: null,
  };
}

export type ChainStatusStore = {
  readonly status: ChainStatus;
  readonly online: boolean;
  readonly reachable: boolean;
  readonly lastUpdated: number | null;
  readonly error: string | null;
};

export function createChainStatusStore(chainId: number = MONAD_CHAIN_ID): ChainStatusStore {
  let status = $state<ChainStatus>(emptyStatus(chainId));
  let lastUpdated = $state<number | null>(null);
  let error = $state<string | null>(null);

  const reachable = $derived(
    status.block_number !== null && status.latency_ms !== null,
  );
  const online = $derived(isTauri() && reachable);

  $effect(() => {
    if (!isTauri()) {
      status = emptyStatus(chainId);
      return;
    }

    let cancelled = false;

    const tick = async () => {
      try {
        const next = await api.meta.chainStatus(chainId);
        if (cancelled) return;
        status = next;
        lastUpdated = Date.now();
        error = null;
      } catch (e) {
        if (cancelled) return;
        status = emptyStatus(chainId);
        error = e instanceof Error ? e.message : String(e);
      }
    };

    tick();
    const id = setInterval(tick, POLL_MS);

    return () => {
      cancelled = true;
      clearInterval(id);
    };
  });

  return {
    get status() { return status; },
    get online() { return online; },
    get reachable() { return reachable; },
    get lastUpdated() { return lastUpdated; },
    get error() { return error; },
  };
}
