//! Output handling module
//!
//! "I've seen it. It's rubbish."

pub mod report;
pub mod state;

pub use report::{parse_severity, write_report};
pub use state::{load_previous, save_current, PreviousState};
