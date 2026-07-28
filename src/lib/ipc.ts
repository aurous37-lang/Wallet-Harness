/**
 * Typed IPC client for the Tauri backend.
 *
 * Every command in `src-tauri/src/commands/` has a matching function here.
 * Types mirror the Rust `Serialize`/`Deserialize` structs. Keep these in
 * sync with the backend — when you add a command, add it in both places.
 *
 * Import pattern:
 *   import { api } from '$lib/ipc';
 *   const info = await api.meta.appInfo();
 */

import { invoke } from '@tauri-apps/api/core';

// ─── meta ─────────────────────────────────────────────────────────────
export interface AppInfo {
  name: string;
  version: string;
  commit: string | null;
  build_mode: 'debug' | 'release';
}

export interface ChainStatus {
  chain_id: number;
  block_number: number | null;
  latency_ms: number | null;
  endpoint_label: string | null;
}

// ─── settings ─────────────────────────────────────────────────────────
export interface RpcEndpoint {
  chain_id: number;
  label: string;
  url: string;
  priority: number;
}

export type ApiKeySlot =
  | 'Alchemy' | 'Infura' | 'QuickNode'
  | 'CoinGecko' | 'DexScreener'
  | 'OneInch' | 'Paraswap';

// ─── wallet / signer ──────────────────────────────────────────────────
export type SignerKind =
  | 'WalletConnectV2'
  | 'LedgerUsb'
  | 'LedgerBluetooth'
  | 'KeystoneQr'
  | 'HotEncryptedKeystore';

export type SignerTransport = 'Local' | 'Usb' | 'Bluetooth' | 'Qr' | 'Relay';

export interface SignerCapabilities {
  autonomous: boolean;
  requires_user_interaction: boolean;
  supports_eip1559: boolean;
  supports_eip712: boolean;
  supports_personal_sign: boolean;
  transport: SignerTransport;
}

export interface SignerOption {
  kind: SignerKind;
  label: string;
  capabilities: SignerCapabilities;
  platform_supported: boolean;
}

export interface ConnectedAccount {
  address: string;
  chain_id: number;
  signer_kind: SignerKind;
}

// ─── portfolio ────────────────────────────────────────────────────────
export interface TokenBalance {
  chain_id: number;
  token_address: string;
  symbol: string;
  decimals: number;
  raw_balance: string;
  ui_balance: number;
}

export interface Position {
  chain_id: number;
  token_address: string;
  symbol: string;
  quantity: number;
  avg_cost_usd: number;
  market_price_usd: number;
  unrealized_pnl_usd: number;
  realized_pnl_usd: number;
  lot_count: number;
}

export interface TxRecord {
  tx_hash: string;
  chain_id: number;
  block_number: number;
  timestamp: number;
  kind: string;
  token_symbol: string | null;
  amount_ui: number | null;
  counterparty: string | null;
}

// ─── market ───────────────────────────────────────────────────────────
export interface Candle {
  t: number; o: number; h: number; l: number; c: number; v: number;
}

export interface SpotPrice {
  chain_id: number;
  token_address: string;
  price_usd: number;
  source: string;
  staleness_ms: number;
}

// ─── orders ───────────────────────────────────────────────────────────
export type OrderStatus =
  | 'Armed' | 'Triggered' | 'AwaitingSignature'
  | 'Submitted' | 'Confirmed' | 'Cancelled' | 'Failed';

export interface Order {
  id: string;
  chain_id: number;
  pair: string;
  kind: string;
  trigger_price_usd: number;
  size_token: number;
  slippage_bps: number;
  status: OrderStatus;
  created_at: number;
}

// ─── swaps ────────────────────────────────────────────────────────────
export interface QuoteResponse {
  router: string;
  amount_out_raw: string;
  price_impact_bps: number;
  estimated_gas: number;
  route_summary: string;
}

export interface UnsignedTx {
  chain_id: number;
  to: string;
  value: string;
  data: string;
  gas_limit: number;
  max_fee_per_gas: string | null;
  max_priority_fee_per_gas: string | null;
  nonce: number;
  intent_id: string;
}

export interface SubmitResponse {
  tx_hash: string;
  submitted_via: string;
}

// ─── api surface ──────────────────────────────────────────────────────
export const api = {
  meta: {
    ping: () => invoke<string>('ping'),
    appInfo: () => invoke<AppInfo>('app_info'),
    chainStatus: (chainId: number) =>
      invoke<ChainStatus>('meta_chain_status', { chainId }),
  },
  settings: {
    setRpcEndpoint: (endpoint: RpcEndpoint) =>
      invoke<void>('set_rpc_endpoint', { endpoint }),
    getRpcEndpoints: (chainId?: number) =>
      invoke<RpcEndpoint[]>('get_rpc_endpoints', { chainId: chainId ?? null }),
    setApiKey: (slot: ApiKeySlot, value: string) =>
      invoke<void>('set_api_key', { slot, value }),
    listApiKeySlots: () =>
      invoke<string[]>('list_api_key_slots'),
  },
  wallet: {
    listSignerTypes: () => invoke<SignerOption[]>('list_signer_types'),
    connect: (kind: SignerKind, transportHint?: string) =>
      invoke<ConnectedAccount>('connect_wallet', {
        req: { kind, transport_hint: transportHint ?? null }
      }),
    activeAccount: () => invoke<ConnectedAccount | null>('active_account'),
    disconnect: () => invoke<void>('disconnect_wallet'),
  },
  portfolio: {
    balances: (address: string) =>
      invoke<TokenBalance[]>('balances', { address }),
    positions: (address: string) =>
      invoke<Position[]>('positions_with_cost_basis', { address }),
    history: (address: string, chainId?: number, limit?: number) =>
      invoke<TxRecord[]>('transaction_history', {
        address, chainId: chainId ?? null, limit: limit ?? null
      }),
  },
  market: {
    candles: (chainId: number, pool: string, timeframeSeconds: number, limit: number) =>
      invoke<Candle[]>('candles', {
        chainId, poolAddress: pool, timeframeSeconds, limit
      }),
    spotPrice: (chainId: number, tokenAddress: string) =>
      invoke<SpotPrice>('spot_price', { chainId, tokenAddress }),
  },
  orders: {
    listOpen: (address: string) =>
      invoke<Order[]>('list_open_orders', { address }),
    armStopLoss: (req: {
      chain_id: number; token_in: string; token_out: string;
      trigger_price_usd: number; size_token: number; slippage_bps: number;
    }) => invoke<string>('arm_stop_loss', { req }),
    armLimit: (req: {
      chain_id: number; token_in: string; token_out: string;
      side: 'buy' | 'sell';
      trigger_price_usd: number; size_token: number; slippage_bps: number;
      prefer_protocol_order: boolean;
    }) => invoke<string>('arm_limit', { req }),
    cancel: (orderId: string) =>
      invoke<void>('cancel_order', { orderId }),
  },
  swaps: {
    quote: (req: {
      chain_id: number; token_in: string; token_out: string;
      amount_in_raw: string; slippage_bps: number;
    }) => invoke<QuoteResponse>('quote', { req }),
    prepare: (req: {
      chain_id: number; from_address: string;
      token_in: string; token_out: string;
      amount_in_raw: string; slippage_bps: number;
      router: string; private_mempool: boolean;
    }) => invoke<UnsignedTx>('prepare_swap', { req }),
    submitSigned: (intentId: string, signedRawTx: string) =>
      invoke<SubmitResponse>('submit_signed_swap', {
        req: { intent_id: intentId, signed_raw_tx: signedRawTx }
      }),
  },
};

/**
 * Detect whether we're running inside a Tauri window. Useful for dev mode
 * where `pnpm dev` in the browser should gracefully degrade instead of
 * throwing on every invoke().
 */
export function isTauri(): boolean {
  // Tauri v2 injects `__TAURI_INTERNALS__` on window.
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}
