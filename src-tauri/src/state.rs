//! Application-wide state shared across Tauri command handlers.
//!
//! Kept intentionally small. Individual subsystems own their own state; this
//! struct just holds handles to them.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};
use tauri::{AppHandle, Manager};

use chain_rpc::provider::{HttpProvider, HttpProviderConfig};
use chain_rpc::RpcRegistry;
use signer::SignerRegistry;

pub struct AppState {
    pub db: SqlitePool,
    pub rpc: Arc<RpcRegistry>,
    pub signers: Arc<SignerRegistry>,
    pub data_dir: PathBuf,
}

impl AppState {
    pub async fn initialize(app: &AppHandle) -> Result<Self> {
        let data_dir = app
            .path()
            .app_data_dir()
            .context("resolving app data dir")?;
        std::fs::create_dir_all(&data_dir).context("creating app data dir")?;

        let db_path = data_dir.join("wallet-harness.db");

        let connect_opts = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal);

        let db = SqlitePoolOptions::new()
            .max_connections(8)
            .connect_with(connect_opts)
            .await
            .context("opening sqlite pool")?;

        sqlx::migrate!("../migrations")
            .run(&db)
            .await
            .context("running migrations")?;

        seed_default_rpc_endpoints(&db)
            .await
            .context("seeding default rpc endpoints")?;

        let rpc = Arc::new(RpcRegistry::new());
        load_rpc_registry(&db, &rpc)
            .await
            .context("loading rpc registry from db")?;

        let signers = Arc::new(SignerRegistry::new());

        Ok(Self {
            db,
            rpc,
            signers,
            data_dir,
        })
    }
}

async fn seed_default_rpc_endpoints(db: &SqlitePool) -> Result<()> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM rpc_endpoints")
        .fetch_one(db)
        .await
        .context("counting rpc_endpoints")?;
    if count > 0 {
        return Ok(());
    }

    let now = chrono::Utc::now().timestamp();
    let defaults: &[(i64, &str, &str, i64)] = &[
        (10143, "Monad Public", "https://testnet-rpc.monad.xyz", 100),
        (1, "Cloudflare", "https://cloudflare-eth.com", 100),
    ];

    for (chain_id, label, url, priority) in defaults {
        sqlx::query(
            "INSERT OR IGNORE INTO rpc_endpoints (chain_id, label, url, priority, enabled, created_at) \
             VALUES (?1, ?2, ?3, ?4, 1, ?5)",
        )
        .bind(*chain_id)
        .bind(*label)
        .bind(*url)
        .bind(*priority)
        .bind(now)
        .execute(db)
        .await
        .context("inserting default rpc endpoint")?;
    }

    Ok(())
}

async fn load_rpc_registry(db: &SqlitePool, rpc: &RpcRegistry) -> Result<()> {
    let rows: Vec<(i64, String, String, i64)> = sqlx::query_as(
        "SELECT chain_id, label, url, priority FROM rpc_endpoints \
         WHERE enabled = 1 ORDER BY chain_id ASC, priority ASC, id ASC",
    )
    .fetch_all(db)
    .await
    .context("loading rpc_endpoints")?;

    for (chain_id, label, url, _priority) in rows {
        let provider = Arc::new(HttpProvider::new(HttpProviderConfig {
            label,
            endpoint: url,
            api_key_header: None,
            timeout_ms: 10_000,
        }));
        rpc.register(chain_id as u64, provider).await;
    }

    Ok(())
}
