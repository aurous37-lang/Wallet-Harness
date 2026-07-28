-- Wallet Harness — initial schema (v0.1.0)
--
-- Design notes:
--   • API keys are NOT stored here. They live in the OS keychain.
--   • The `rpc_endpoints` table stores URLs and labels only; any API-key
--     material embedded in a URL template is a placeholder, resolved at
--     call time from the keychain.
--   • All monetary / quantity values use TEXT for u256 safety (decimal-
--     stringified where appropriate). Aggregated UI-friendly floats are
--     computed at read time.
--   • Every table uses `created_at` / `updated_at` in unix seconds so the
--     app stays timezone-agnostic; display-time conversion is the UI's job.

PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;

CREATE TABLE IF NOT EXISTS rpc_endpoints (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    chain_id    INTEGER NOT NULL,
    label       TEXT    NOT NULL,
    url         TEXT    NOT NULL,
    priority    INTEGER NOT NULL DEFAULT 100,
    enabled     INTEGER NOT NULL DEFAULT 1,
    created_at  INTEGER NOT NULL,
    UNIQUE(chain_id, label)
);
CREATE INDEX IF NOT EXISTS idx_rpc_chain ON rpc_endpoints(chain_id, priority);

CREATE TABLE IF NOT EXISTS accounts (
    address         TEXT PRIMARY KEY,       -- checksummed
    label           TEXT,
    signer_kind     TEXT NOT NULL,          -- "WalletConnectV2" | "LedgerUsb" | ...
    added_at        INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS tokens (
    chain_id        INTEGER NOT NULL,
    address         TEXT    NOT NULL,       -- lowercased; "native" for gas token
    symbol          TEXT    NOT NULL,
    name            TEXT,
    decimals        INTEGER NOT NULL,
    logo_uri        TEXT,
    PRIMARY KEY (chain_id, address)
);

CREATE TABLE IF NOT EXISTS pools (
    chain_id        INTEGER NOT NULL,
    address         TEXT    NOT NULL,
    dex_kind        TEXT    NOT NULL,       -- "UniswapV2" | "UniswapV3" | "MonadNative"
    token0          TEXT    NOT NULL,
    token1          TEXT    NOT NULL,
    fee_bps         INTEGER,
    label           TEXT,
    backfilled_to   INTEGER NOT NULL DEFAULT 0,  -- last indexed block
    PRIMARY KEY (chain_id, address)
);
CREATE INDEX IF NOT EXISTS idx_pool_pair ON pools(chain_id, token0, token1);

CREATE TABLE IF NOT EXISTS swaps (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    chain_id        INTEGER NOT NULL,
    pool_address    TEXT    NOT NULL,
    block_number    INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    tx_hash         TEXT    NOT NULL,
    log_index       INTEGER NOT NULL,
    sender          TEXT    NOT NULL,
    recipient       TEXT    NOT NULL,
    amount0_in      TEXT    NOT NULL,       -- u256 stringified
    amount1_in      TEXT    NOT NULL,
    amount0_out     TEXT    NOT NULL,
    amount1_out     TEXT    NOT NULL,
    price_derived   REAL,                   -- token1/token0 at swap, nullable
    UNIQUE(chain_id, tx_hash, log_index)
);
CREATE INDEX IF NOT EXISTS idx_swaps_pool_time ON swaps(chain_id, pool_address, block_timestamp);

CREATE TABLE IF NOT EXISTS transactions (
    chain_id        INTEGER NOT NULL,
    tx_hash         TEXT    NOT NULL,
    block_number    INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    from_addr       TEXT    NOT NULL,
    to_addr         TEXT,
    value           TEXT    NOT NULL,       -- u256
    gas_used        INTEGER,
    effective_gas_price TEXT,
    status          INTEGER NOT NULL,       -- 1 = success, 0 = reverted
    kind            TEXT    NOT NULL,       -- "swap" | "transfer_in" | "transfer_out" | "approval" | "other"
    PRIMARY KEY (chain_id, tx_hash)
);
CREATE INDEX IF NOT EXISTS idx_tx_from ON transactions(chain_id, from_addr, block_timestamp);
CREATE INDEX IF NOT EXISTS idx_tx_to   ON transactions(chain_id, to_addr,   block_timestamp);

-- Cost-basis lots (FIFO).
CREATE TABLE IF NOT EXISTS lots (
    id              TEXT PRIMARY KEY,       -- uuid
    account         TEXT NOT NULL,
    chain_id        INTEGER NOT NULL,
    token_address   TEXT NOT NULL,
    quantity        REAL NOT NULL,          -- remaining (after partial disposals)
    quantity_original REAL NOT NULL,
    unit_cost_usd   REAL NOT NULL,
    acquired_at     INTEGER NOT NULL,
    source_tx_hash  TEXT NOT NULL,
    closed          INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_lots_open ON lots(account, chain_id, token_address, acquired_at)
    WHERE closed = 0;

CREATE TABLE IF NOT EXISTS realized_gains (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    lot_id          TEXT NOT NULL,
    account         TEXT NOT NULL,
    chain_id        INTEGER NOT NULL,
    token_address   TEXT NOT NULL,
    quantity        REAL NOT NULL,
    cost_basis_usd  REAL NOT NULL,
    proceeds_usd    REAL NOT NULL,
    gain_usd        REAL NOT NULL,
    holding_days    INTEGER NOT NULL,
    disposed_at     INTEGER NOT NULL,
    source_tx_hash  TEXT NOT NULL,
    FOREIGN KEY (lot_id) REFERENCES lots(id)
);
CREATE INDEX IF NOT EXISTS idx_gains_account ON realized_gains(account, disposed_at);

-- Armed orders (stop-loss / limit).
CREATE TABLE IF NOT EXISTS orders (
    id              TEXT PRIMARY KEY,       -- uuid
    account         TEXT NOT NULL,
    chain_id        INTEGER NOT NULL,
    kind            TEXT NOT NULL,          -- "stop_loss" | "limit_buy" | "limit_sell"
    token_in        TEXT NOT NULL,
    token_out       TEXT NOT NULL,
    size_token      REAL NOT NULL,
    trigger_price_usd REAL NOT NULL,
    slippage_bps    INTEGER NOT NULL,
    state           TEXT NOT NULL,          -- "Armed" | "Triggered" | "AwaitingSignature" | "Submitted" | "Confirmed" | "Cancelled" | "Failed"
    autonomous_allowed INTEGER NOT NULL DEFAULT 0, -- v2-only; ignored in v1
    prefer_protocol_order INTEGER NOT NULL DEFAULT 0,
    tx_hash         TEXT,                   -- populated once submitted
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL,
    triggered_at    INTEGER,
    resolved_at     INTEGER
);
CREATE INDEX IF NOT EXISTS idx_orders_open ON orders(account, state, chain_id)
    WHERE state IN ('Armed', 'Triggered', 'AwaitingSignature', 'Submitted');

-- Candle cache (optional acceleration layer; derivable from swaps).
CREATE TABLE IF NOT EXISTS candles_cache (
    chain_id        INTEGER NOT NULL,
    pool_address    TEXT NOT NULL,
    timeframe_s     INTEGER NOT NULL,
    bucket_ts       INTEGER NOT NULL,
    o REAL, h REAL, l REAL, c REAL, v REAL,
    PRIMARY KEY (chain_id, pool_address, timeframe_s, bucket_ts)
);

-- Slot registry for API keys stored in keychain. Values live in keychain;
-- this table only tracks which slots are configured.
CREATE TABLE IF NOT EXISTS api_key_slots (
    slot_name       TEXT PRIMARY KEY,       -- "Alchemy" | "Infura" | "CoinGecko" | ...
    configured_at   INTEGER NOT NULL
);
