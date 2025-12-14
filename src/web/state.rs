//! Shared application state for web server

use chrono::{DateTime, Utc};
use tokio::sync::Mutex;

use crate::config::Config;

/// Application state shared across all web request handlers
pub struct AppState {
    /// Application configuration
    pub config: Config,

    /// Mutex to prevent concurrent collections
    /// When locked, a collection is in progress
    pub collection_lock: Mutex<()>,

    /// Timestamp of last collection run
    pub last_run: Mutex<Option<DateTime<Utc>>>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            collection_lock: Mutex::new(()),
            last_run: Mutex::new(None),
        }
    }
}
