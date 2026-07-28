//! Tauri IPC command handlers.
//!
//! Each submodule exposes a narrow, typed surface that the frontend calls
//! via `invoke()`. Commands return `Result<T, String>` so errors serialize
//! cleanly to the frontend.

pub mod market;
pub mod meta;
pub mod orders;
pub mod portfolio;
pub mod settings;
pub mod swaps;
pub mod wallet;
