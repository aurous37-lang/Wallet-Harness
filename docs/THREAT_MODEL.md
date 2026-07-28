# Threat Model & Privacy Principles

## Trust boundary

The Wallet Harness trust boundary is the user's device. Everything inside
the app — the SQLite database, the OS keychain entries, the local indexer
state — is treated as trusted storage. Everything outside the device is
treated as hostile by default.

The user is **not** a threat to themselves. Footguns exist (e.g. arming a
stop-loss with bad slippage) but the app should inform rather than
restrict. Confirmation dialogs for destructive or irreversible actions,
yes. Paternalistic gates, no.

## Non-goals (v1)

- **Key custody.** The app never holds private keys. Every signature comes
  from a user-chosen signer: WalletConnect v2, Ledger, or Keystone.
- **Telemetry.** No analytics, no crash reporters, no remote logging.
  `tracing` writes to stderr only.
- **Account linking.** The app does not know who the user is. It knows
  addresses, which it reads off-chain + on-chain data for. No email, no
  phone, no KYC, no pseudonymous identifiers.

## Data flows

### What never leaves the device
- Private keys (the app never has them)
- The SQLite database
- OS keychain entries
- `tracing` logs
- Any file under `~/.local/share/wallet-harness/` (or OS equivalent)

### What leaves the device only on explicit user action
- RPC calls to the user-configured endpoint(s)
- Price queries to DexScreener / CoinGecko (only if user configures keys)
- Swap quote requests to 1inch / Paraswap / 0x (only if user enables)
- Transaction broadcasts to the selected mempool (public or private)
- Signature-approval traffic over the WalletConnect relay (end-to-end
  encrypted; relay sees no payload)

Every external surface is enumerated in `src-tauri/tauri.conf.json`'s CSP
`connect-src` directive. Adding a new external host requires editing that
file, which is a review gate.

## Keychain usage

API keys are stored in the OS keychain under service name
`wallet-harness` with slot-named entries. Backends:
- Linux: Secret Service (libsecret) — GNOME Keyring / KWallet
- Windows: Credential Manager
- macOS: Keychain
- Android: Keystore (via Tauri v2 mobile bridge)

The SQLite `api_key_slots` table records **only slot names**, never values.
This is intentional: a leaked SQLite file reveals which services the user
has configured but not the key material itself.

## Address privacy

The app displays wallet addresses to the user. It does not submit them to
any external API beyond the user's chosen RPC endpoints and (opt-in)
aggregator APIs. Price queries for a token do not include the user's
address. Only swap-related calls (1inch quote, broadcast) include it,
because chain semantics require it.

## MEV exposure

Every user-initiated swap carries the option to submit via a private
mempool (Flashbots Protect on Ethereum; chain-specific equivalents
elsewhere). The default is public mempool because (a) it's the only option
on many chains, and (b) for small swaps the privacy benefit isn't worth
the hit rate penalty. Users with large orders should toggle on private
submission.

## What the threat model does *not* cover

- A compromised host OS can read anything the app reads. Don't run the
  app on a compromised machine. This is true of every application.
- A compromised WalletConnect wallet can sign malicious transactions. The
  app shows the decoded transaction before requesting signature, but
  ultimately the signer's own display is the source of truth.
- Supply-chain attacks against Rust/npm dependencies. Mitigated by
  `Cargo.lock` / `pnpm-lock.yaml` pinning and periodic audits.
