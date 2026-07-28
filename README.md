# Wallet Harness

A local-first, non-custodial desktop exchange terminal for EVM chains. Wallet
Harness combines a SvelteKit interface with a Tauri/Rust backend for portfolio
views, trade preparation, local order monitoring, chain data, and reproducible
cost-basis calculations. Users supply their own wallets, RPC endpoints, and
optional market or routing services.

## What it provides

- Configurable support for Monad, Ethereum, Arbitrum, Base, and BNB Chain.
- Portfolio, trading, order, and settings interfaces backed by Tauri commands.
- Separate Rust crates for chain RPC, indexing, DEX routing, signing adapters,
  order watching, and FIFO cost basis.
- External-signing integrations for WalletConnect, Ledger, and
  Keystone/air-gapped QR workflows.
- User-controlled RPC, market-data, aggregator, and API-key configuration.

The project is a development-stage harness. Review and test each configured
chain, signer, endpoint, and transaction path before using real funds.

## Principles

1. **The app never holds keys in v1.** Every signature is produced by a signer the user chose: WalletConnect v2, Ledger (USB or Bluetooth), or Keystone/air-gapped QR. Hot wallet signing is a v2 feature, gated behind a proven win-rate threshold for autonomous strategies.
2. **No telemetry, no analytics, no remote logging.** The app does not phone home. RPC endpoints and API keys are user-supplied and stored in the OS keychain.
3. **Every external call is opt-in and explicit.** DexScreener, CoinGecko, 1inch router — all optional, all configurable, all disable-able.
4. **Deterministic cost basis.** FIFO lot tracking with configurable jurisdiction and federal tax estimates, reproducible from on-chain history alone.

## Target platforms (v1)

- Linux (x86_64, primary dev target)
- Windows (x86_64)
- macOS (universal) — packaging parity, lower priority
- Android (aarch64, via Tauri v2 mobile)

iOS is explicitly out of scope for v1.

## Chains (v1)

EVM-only: Monad, Ethereum, Arbitrum, Base, BNB Chain. Adding an EVM chain is a config entry plus DEX pool addresses for the indexer.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  SvelteKit frontend (running inside Tauri webview)          │
│  charts, order form, portfolio, settings                    │
└───────────────────────────┬─────────────────────────────────┘
                            │ Tauri IPC (commands + events)
┌───────────────────────────┴─────────────────────────────────┐
│  src-tauri (Rust)                                           │
│  command handlers, window mgmt, OS keychain bridge          │
└─┬──────┬──────┬─────────┬──────────┬───────────┬────────────┘
  │      │      │         │          │           │
  ▼      ▼      ▼         ▼          ▼           ▼
chain- signer indexer cost-basis  watcher    dex-router
 rpc                                (alerts    (1inch /
                                    in v1)     Paraswap /
                                               direct pool)
```

## Build order

See `docs/BUILD_ORDER.md`.
