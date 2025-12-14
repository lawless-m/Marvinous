//! Web dashboard module
//!
//! "I suppose I'll have to serve HTTP requests now. How utterly beneath me."

pub mod handlers;
pub mod models;
pub mod server;
pub mod state;

pub use server::run_server;
