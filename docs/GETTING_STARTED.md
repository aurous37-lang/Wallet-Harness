# Getting Started

## Prerequisites

- **Rust** ≥ 1.78 (`rustup default stable`)
- **Node.js** ≥ 20 + **pnpm** ≥ 9 (`npm i -g pnpm`)
- **Tauri v2 prerequisites** for your OS — see https://v2.tauri.app/start/prerequisites/
  - Linux: `libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`, plus
    `build-essential`, `curl`, `wget`, `file`, `libssl-dev`, `libgtk-3-dev`,
    `libsoup-3.0-dev`, `libjavascriptcoregtk-4.1-dev`.
  - Windows: Microsoft C++ Build Tools + WebView2 (ships with modern Win11).
  - macOS: Xcode Command Line Tools.
- **Android** (for step 10 later): Android Studio + NDK, set `ANDROID_HOME`
  and `NDK_HOME`. Tauri's `tauri android init` will scaffold the Gradle
  project when you're ready.

## First run

```bash
# install frontend deps
pnpm install

# run in dev mode — opens the Tauri window with HMR on the SvelteKit side
pnpm tauri dev
```

On first launch the app creates its SQLite database at the OS-appropriate
app-data directory and runs migrations. You should see the overview page
with "ONLINE" in the hero panel, meaning the backend `ping` command
responded.

## Browser-only dev mode

If you just want to iterate on the frontend without building the Rust side:

```bash
pnpm dev
```

This opens the SvelteKit app at http://localhost:5173. The app will detect
it's not running inside Tauri and display a warning banner; all IPC calls
become no-ops. Useful for pure CSS/component work.

## Building a release

```bash
# desktop bundle for your current platform
pnpm tauri build
```

Output goes to `src-tauri/target/release/bundle/`.

## Backend dev workflow

If you're working on the Rust crates specifically and don't need the full
app loop:

```bash
# type-check the whole workspace
cargo check --workspace

# run tests for a single crate
cargo test -p cost-basis
```

## Directory map

```
wallet-harness/
├── Cargo.toml              # workspace manifest
├── package.json            # frontend deps
├── src/                    # SvelteKit frontend
│   ├── app.css             # design tokens, global styles
│   ├── app.html            # HTML shell
│   ├── lib/
│   │   ├── ipc.ts          # typed Tauri command client
│   │   └── components/     # Topbar, Sidebar, Statusbar
│   └── routes/             # pages (overview, portfolio, trade, orders, settings)
├── src-tauri/              # Tauri app (Rust)
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs          # entry + command registration
│   │   ├── state.rs        # shared app state
│   │   └── commands/       # IPC handlers
│   ├── tauri.conf.json
│   └── capabilities/
├── crates/                 # backend crates
│   ├── chain-rpc/          # multi-provider EVM RPC with failover
│   ├── signer/             # Signer trait + WC/Ledger/Keystone impls
│   ├── indexer/            # local event indexer + OHLCV candles
│   ├── cost-basis/         # FIFO lot tracker + tax awareness
│   ├── watcher/            # armed-order engine
│   └── dex-router/         # swap routing (aggregators + direct)
├── migrations/             # sqlx migrations
└── docs/
    ├── BUILD_ORDER.md      # step-by-step v1 plan
    └── THREAT_MODEL.md     # privacy + security principles
```

## Adding a chain

Three touch points, all in the `chain-rpc` crate:
1. Add a variant to `ChainId` in `chains.rs`
2. Add a branch to `meta()` returning the `ChainMeta`
3. Users then add RPC endpoints via `settings → rpc endpoints`

## Adding a signer

Implement `signer::Signer` for your new backend and add a variant to
`SignerKind`. Register it in `src-tauri/src/commands/wallet.rs` under
`list_signer_types`. The `SignerCapabilities` descriptor drives what the
UI offers — set `autonomous: false` for anything that requires user
interaction per-tx.
