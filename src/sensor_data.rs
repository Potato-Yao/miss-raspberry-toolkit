//! Cached sensor readings fetched from the [`multimeter_engine`] backend.
//!
//! [`SensorData::refresh`] pulls the latest values from the engine's
//! in-memory map.  The struct is designed to be called once per frame
//! and shared between the sidebar and dashboard.

use multimeter_engine::monitor::{self, QueryRequest};
use multimeter_engine::util::data_container::DataContainer;

/// All sensor readings the UI cares about, pre-formatted as [`Option<String>`].
///
/// `None` means the value is not (yet) available.
#[derive(Default, Clone, Debug)]
pub struct SensorData {
    // ── CPU ──────────────────────────────────────────────────────
    pub cpu_name: Option<String>,
    pub cpu_temperature: Option<f64>,
    pub cpu_power: Option<f64>,
    pub cpu_voltage: Option<f64>,
    pub cpu_clock: Option<f64>,
    pub cpu_usage: Option<f64>,

    // ── GPU ──────────────────────────────────────────────────────
    pub gpu_name: Option<String>,
    pub gpu_temperature: Option<f64>,
    pub gpu_power: Option<f64>,
    pub gpu_clock: Option<f64>,
    pub gpu_usage: Option<f64>,

    // ── Memory ───────────────────────────────────────────────────
    pub mem_used: Option<f64>,
    pub mem_available: Option<f64>,
    pub mem_percentage: Option<f64>,

    // ── Battery ──────────────────────────────────────────────────
    pub bat_capacity_designed: Option<f64>,
    pub bat_capacity_max: Option<f64>,
    pub bat_capacity_remain: Option<f64>,
    pub bat_rate: Option<f64>,
    pub bat_state: Option<String>,
    pub bat_voltage: Option<f64>,

    // ── Disk ─────────────────────────────────────────────────────
    pub disk_sizes: Option<String>,
    pub disk_partitions: Option<String>,
    pub disk_partition_detail: Option<String>,

    // ── System ───────────────────────────────────────────────────
    pub os_activated: Option<String>,
}

impl SensorData {
    /// Pull the latest values from the engine.
    ///
    /// This is cheap — it only reads a `Mutex<HashMap>` that is updated
    /// by the engine's background threads.  Call once per frame.
    pub fn refresh(&mut self) {
        // ── CPU ──────────────────────────────────────────────────
        self.cpu_name = query_string("cpu_name");
        self.cpu_temperature = query_f64("cpu_temperature");
        self.cpu_power = query_f64("cpu_power");
        self.cpu_voltage = query_f64("cpu_voltage");
        self.cpu_clock = query_f64("cpu_clock_avg");
        self.cpu_usage = query_f64("cpu_usage");

        // ── GPU ──────────────────────────────────────────────────
        self.gpu_name = query_string("gpu_name");
        self.gpu_temperature = query_f64("gpu_temperature");
        self.gpu_power = query_f64("gpu_power");
        self.gpu_clock = query_f64("gpu_clock_rms");
        self.gpu_usage = query_f64("gpu_usage");

        // ── Memory ───────────────────────────────────────────────
        self.mem_used = query_f64("mem_used");
        self.mem_available = query_f64("mem_available");
        self.mem_percentage = query_f64("mem_percentage");

        // ── Battery ──────────────────────────────────────────────
        self.bat_capacity_designed = query_f64("bat_capacity_designed");
        self.bat_capacity_max = query_f64("bat_capacity_max");
        self.bat_capacity_remain = query_f64("bat_capacity_remain");
        self.bat_rate = query_f64("bat_rate");
        self.bat_state = query_string("bat_state");
        self.bat_voltage = query_f64("bat_voltage");

        // ── Disk ─────────────────────────────────────────────────
        self.disk_sizes = query_string("disk_disk_size");
        self.disk_partitions = query_string("disk_partition");
        self.disk_partition_detail = query_string("disk_partition_detail");

        // ── System ───────────────────────────────────────────────
        self.os_activated = query_string("os_activated");
    }
}

// ── Formatting helpers ──────────────────────────────────────────

const PLACEHOLDER: &str = "---";

/// Format an `Option<f64>` with a fixed number of decimal places and a unit.
pub fn fmt_value(value: Option<f64>, decimals: usize, unit: &str) -> String {
    match value {
        Some(v) => format!("{v:.decimals$}{unit}"),
        None => format!("{PLACEHOLDER}{unit}"),
    }
}

/// Format an `Option<String>`, falling back to `"---"`.
pub fn fmt_text(value: &Option<String>) -> &str {
    value.as_deref().unwrap_or(PLACEHOLDER)
}

/// Battery percentage derived from remain / max capacity.
pub fn bat_health_pct(data: &SensorData) -> Option<f64> {
    match (data.bat_capacity_remain, data.bat_capacity_max) {
        (Some(remain), Some(max)) if max > 0.0 => Some(remain / max * 100.0),
        _ => None,
    }
}

// ── Private query helpers ───────────────────────────────────────

/// Issue a single query and return the raw `DataContainer`.
fn query_raw(target: &str) -> Option<DataContainer> {
    let request = QueryRequest {
        target: target.to_owned(),
        parameter: None,
    };
    monitor::query_info(request).ok().map(|p| p.value)
}

/// Query a value and try to extract it as `f64`.
fn query_f64(target: &str) -> Option<f64> {
    query_raw(target).and_then(|dc| match dc {
        DataContainer::Float(v) => Some(v),
        DataContainer::Int(v) => Some(f64::from(v)),
        _ => None,
    })
}

/// Query a value and convert it to `String`.
fn query_string(target: &str) -> Option<String> {
    query_raw(target).map(String::from)
}

