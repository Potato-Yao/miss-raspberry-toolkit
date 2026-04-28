//! Cached sensor readings fetched from the [`multimeter_engine`] backend.
//!
//! [`SensorData::refresh`] pulls the latest values from the engine's
//! in-memory map.  The struct is designed to be called once per frame
//! and shared between the sidebar and dashboard.

use multimeter_engine::monitor::{self, QueryRequest};
use multimeter_engine::util::data_container::DataContainer;

/// Disk information deserialized from the engine's `"disk_disk"` query.
///
/// Each item in the `DataContainer::Array` is a JSON string matching this shape.
#[derive(Clone, Debug, Default, serde::Deserialize)]
pub struct DiskInfo {
    /// Disk kind – `"SSD"`, `"HDD"`, or `"Unknown"`.
    #[serde(rename = "DiskKind", default)]
    pub kind: String,
    /// Device name, e.g. `"/dev/nvme0n1p1"`.
    #[serde(default)]
    pub name: String,
    /// Filesystem type, e.g. `"ext4"`.
    #[serde(default)]
    pub file_system: String,
    /// Mount point path, e.g. `"/"`.
    #[serde(default)]
    pub mount_point: String,
    /// Total space in bytes.
    #[serde(default)]
    pub total_space: u64,
    /// Available (free) space in bytes.
    #[serde(default)]
    pub available_space: u64,
    /// Whether the disk is removable.
    #[serde(default)]
    pub is_removable: bool,
}

impl DiskInfo {
    /// Used space in bytes.
    pub fn used_space(&self) -> u64 {
        self.total_space.saturating_sub(self.available_space)
    }

    /// Usage percentage (0–100).
    pub fn usage_pct(&self) -> f64 {
        if self.total_space == 0 {
            return 0.0;
        }
        self.used_space() as f64 / self.total_space as f64 * 100.0
    }
}

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
    pub mem_total: Option<f64>,
    pub mem_used: Option<f64>,
    pub mem_available: Option<f64>,
    pub mem_percentage: Option<f64>,
    pub mem_swap_total: Option<f64>,
    pub mem_swap_used: Option<f64>,

    // ── Battery ──────────────────────────────────────────────────
    pub bat_capacity_designed: Option<f64>,
    pub bat_capacity_max: Option<f64>,
    pub bat_capacity_remain: Option<f64>,
    pub bat_rate: Option<f64>,
    pub bat_state: Option<String>,
    pub bat_voltage: Option<f64>,

    // ── Disk ─────────────────────────────────────────────────────
    pub disks: Vec<DiskInfo>,

    // ── System ───────────────────────────────────────────────────
    pub os_name: Option<String>,
    pub os_activated: Option<String>,
    pub os_kernel_version: Option<String>,
    pub os_version: Option<String>,
    pub os_host_name: Option<String>,

    pub fan_cpu: Option<i32>,
    pub fan_gpu: Option<i32>,
    pub fan_mid: Option<i32>,
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
        self.cpu_clock = query_f64("cpu_clock_rms");
        self.cpu_usage = query_f64("cpu_usage");

        // ── GPU ──────────────────────────────────────────────────
        self.gpu_name = query_string("gpu_name");
        self.gpu_temperature = query_f64("gpu_temperature");
        self.gpu_power = query_f64("gpu_power");
        self.gpu_clock = query_f64("gpu_clock_rms");
        self.gpu_usage = query_f64("gpu_usage");

        // ── Memory ───────────────────────────────────────────────
        self.mem_total = query_f64("mem_total");
        self.mem_used = query_f64("mem_used");
        self.mem_available = query_f64("mem_available");
        self.mem_percentage = query_f64("mem_percentage");
        self.mem_swap_total = query_f64("mem_swap_total");
        self.mem_swap_used = query_f64("mem_swap_used");

        // ── Battery ──────────────────────────────────────────────
        self.bat_capacity_designed = query_f64("bat_capacity_designed");
        self.bat_capacity_max = query_f64("bat_capacity_max");
        self.bat_capacity_remain = query_f64("bat_capacity_remain");
        self.bat_rate = query_f64("bat_rate");
        self.bat_state = query_string("bat_state");
        self.bat_voltage = query_f64("bat_voltage");

        // ── Disk ─────────────────────────────────────────────────
        self.disks = query_disk_array("disk_disk");

        // ── System ───────────────────────────────────────────────
        self.os_name = query_string("os_name");
        self.os_activated = query_string("os_activated");
        self.os_kernel_version = query_string("os_kernel_version");
        self.os_version = query_string("os_version");
        self.os_host_name = query_string("os_host_name");

        self.fan_cpu = query_i32("fan_rpm_cpu");
        self.fan_gpu = query_i32("fan_rpm_gpu");
        self.fan_mid = query_i32("fan_rpm_mid");
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

/// Format a byte count as a human-readable string (e.g. `"838.9 GB"`).
pub fn fmt_bytes(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;
    const TIB: f64 = GIB * 1024.0;
    let b = bytes as f64;
    if b >= TIB {
        format!("{:.1} TB", b / TIB)
    } else if b >= GIB {
        format!("{:.1} GB", b / GIB)
    } else if b >= MIB {
        format!("{:.1} MB", b / MIB)
    } else if b >= KIB {
        format!("{:.1} KB", b / KIB)
    } else {
        format!("{bytes} B")
    }
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
        DataContainer::UnsignedLong(v) => Some(v as f64),
        _ => None,
    })
}

fn query_i32(target: &str) -> Option<i32> {
    query_raw(target).and_then(|dc| match dc {
        DataContainer::Int(v) => Some(v),
        _ => None,
    })
}

/// Query a value and convert it to `String`.
fn query_string(target: &str) -> Option<String> {
    query_raw(target).map(String::from)
}

/// Query a `DataContainer::Array` of JSON strings and deserialize each into [`DiskInfo`].
fn query_disk_array(target: &str) -> Vec<DiskInfo> {
    let Some(DataContainer::Array(items)) = query_raw(target) else {
        return Vec::new();
    };
    items
        .into_iter()
        .filter_map(|dc| {
            if let DataContainer::Text(json) = dc {
                serde_json::from_str::<DiskInfo>(&json).ok()
            } else {
                None
            }
        })
        .collect()
}
