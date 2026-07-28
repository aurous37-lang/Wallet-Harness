# Build Order — Wallet Harness v1

This is the shipping plan. Ten steps, each a discrete checkpoint. Land one
before starting the next; the order is chosen so every step has a working
demo and nothing waits on a dependency that hasn't been built yet.

## 01 · Scaffold & signer abstraction — ✅ complete

Cargo workspace, Tauri v2 config, SQLite migrations, `Signer` trait with
`WalletConnectV2` / `LedgerUsb` / `LedgerBluetooth` / `KeystoneQr` variants,
typed IPC surface, SvelteKit frontend shell with terminal aesthetic.

**Exit criteria:** `pnpm tauri dev` opens the app window, backend responds
to `ping`, all routes render placeholder scaffolds.

## 02 · Chain-RPC + Monad read path

Implement `HttpProvider::raw_request` with reqwest. Wire the registry's
failover logic end-to-end. Implement `chain_id`, `block_number`,
`native_balance`, `get_logs`, `send_raw_transaction` on the registry.
Monad mainnet first (you know it best; your CrabTrap wallet is the test
fixture).

**Exit criteria:** statusbar shows live block height and latency for Monad.
`portfolio.balances(your_address)` returns real native + ERC-20 balances.

## 03 · Cost-basis port (FIFO, configurable jurisdiction rate)

Port your existing FIFO module into the `cost-basis` crate. Back it with
SQLite (`lots`, `realized_gains` tables). Feed it from indexer events
(step 4). Until step 4 lands, use a CSV seeder that replays your known
history for testing.

**Exit criteria:** positions panel populates with average cost, realized +
unrealized P&L. Numbers match your existing FIFO module output to the cent.

## 04 · Charts + pricing

Stand up the indexer: chunked `eth_getLogs` backfill of Uniswap V2-style
Swap events for a handful of Monad pools you care about. Derive OHLCV
candles on the fly. Wire TradingView Lightweight Charts to the
`market.candles` IPC endpoint. Add DexScreener fallback with a
user-supplied API key.

**Exit criteria:** real candles for MON/USDC (or the pair you trade) on the
trade page. Toggle between indexer-derived and DexScreener-derived source.

## 05 · Manual swaps via WalletConnect

Implement `WalletConnectSigner::pair_qr` and `pair_deeplink`. Implement the
three-step swap flow: `quote → prepare_swap → submit_signed_swap`. Start
with direct Uniswap V2 routing (in-process math) for Monad; add 1inch for
Ethereum. Private-mempool toggle for ETH swaps via Flashbots Protect RPC.

**Exit criteria:** you can execute a swap from MetaMask Mobile on the
desktop app via WC v2 and see it land on chain, with cost-basis updating
automatically from the new tx.

## 06 · Armed-order engine (no autonomous)

Implement the watcher state machine: `Armed → Triggered →
AwaitingSignature → Submitted → Confirmed | Failed`. Multi-source price
consensus (2-of-N, bounded dispersion). Emit Tauri events on state
transitions. Orders page subscribes and updates the kanban live. Desktop
notifications on `Triggered`.

**Exit criteria:** arm a stop-loss on a small position, watch it transition
to `Triggered` when price crosses, get a notification, open the app, approve
the signature through WC, see it resolve to `Confirmed`.

## 07 · Hardware signer (manual orders only)

Implement `LedgerSigner::connect` for USB HID on desktop. APDU protocol
for the Ethereum app. EIP-1559 + EIP-712 support. Bluetooth can wait for
step 10 with Android.

**Exit criteria:** same swap flow as step 5, but signing via a physical
Ledger device.

## 08 · Multi-chain portfolio view

Ethereum mainnet + Arbitrum + Base + BNB. Validates that the chain
abstraction actually abstracts. Portfolio view aggregates across chains
with per-chain filtering. Indexer runs one worker per configured chain.

**Exit criteria:** a single portfolio panel showing positions on 3+ chains,
with cost basis reproducible from on-chain history for each.

## 09 · Windows + macOS packaging

Bundle MSI (Windows), DMG (macOS), DEB + AppImage (Linux — already in
`tauri.conf.json`). Code-signing where possible. Keychain path differences
per platform validated.

**Exit criteria:** installable binaries for all three desktop platforms.
CI workflow that produces them.

## 10 · Android companion

Tauri v2 Android target. Read view + charts + cost basis + manual swaps
via WalletConnect (deep-link, not QR, since app is on-device). Ledger
Bluetooth optional. Keystone QR via native camera. On-chain limit orders
work trivially since the app doesn't need to stay running.

**Exit criteria:** signed APK. Companion app shows real portfolio, opens
MetaMask Mobile via deep-link to sign swaps, receives push notifications
when armed orders trigger.

---

## V2 gate — autonomous execution

Unlocks when a strategy (likely CrabTrap or its successor) demonstrates a
sustained ≥ 73% live win rate over a defined evaluation window. At that
point:

- `HotEncryptedKeystore` signer becomes offerable
- Orders gain an `autonomous_allowed` flag (schema already reserves it)
- The watcher state machine gains a direct `Triggered → Submitted`
  transition, bypassing `AwaitingSignature`, for orders with the flag set
- An audit log surfaces every autonomous action for review

Until then, the app enforces human-in-the-loop on every signature.
