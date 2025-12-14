//! Data collectors for Marvinous
//!
//! "I have a million ideas, but they all point to certain death."

pub mod ipmi;
pub mod journalctl;
pub mod nvidia;
pub mod sensors;
pub mod smart;

pub use ipmi::{collect_ipmi, IpmiReading};
pub use journalctl::{collect_kernel_logs, collect_system_logs, LogEntry};
pub use nvidia::{collect_gpu, GpuStatus};
pub use sensors::{collect_sensors, SensorReading};
pub use smart::{collect_smart, DriveHealth};

use serde::{Deserialize, Serialize};

use crate::output::state::PreviousState;

/// All collected data from a single run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectedData {
    pub system_logs: Vec<LogEntry>,
    pub kernel_logs: Vec<LogEntry>,
    pub sensors: Vec<SensorReading>,
    pub ipmi: Vec<IpmiReading>,
    pub gpu: Option<GpuStatus>,
    pub drives: Vec<DriveHealth>,
    pub previous: Option<PreviousState>,
}
