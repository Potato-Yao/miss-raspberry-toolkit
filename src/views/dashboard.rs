use super::card::{CardPanel, CardWidth};
use crate::sensor_data::{SensorData, bat_health_pct, fmt_bytes, fmt_text, fmt_value};
use rust_i18n::t;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct DashboardView;

/// Fixed card height used on the dashboard.
const CARD_HEIGHT: f32 = 200.0;

/// Placeholder shown when a value is not yet available.
const DEFAULT_VALUE: &str = "---";

impl DashboardView {
    pub fn ui(&mut self, ui: &mut egui::Ui, data: &SensorData) {
        CardPanel::show(ui, CARD_HEIGHT, |panel, ui| {
            panel.card(ui, &t!("card_cpu"), CardWidth::Half, |ui| {
                info_row(
                    ui,
                    &t!("label_temperature"),
                    &fmt_value(data.cpu_temperature, 1, "°C"),
                );
                info_row(ui, &t!("label_power"), &fmt_value(data.cpu_power, 1, "W"));
                info_row(
                    ui,
                    &t!("label_voltage"),
                    &fmt_value(data.cpu_voltage, 3, "V"),
                );
                info_row(ui, &t!("label_clock"), &fmt_value(data.cpu_clock, 0, "MHz"));
                info_row(ui, &t!("label_usage"), &fmt_value(data.cpu_usage, 0, "%"));
            });

            panel.card(ui, &t!("card_gpu"), CardWidth::Half, |ui| {
                info_row(
                    ui,
                    &t!("label_temperature"),
                    &fmt_value(data.gpu_temperature, 1, "°C"),
                );
                info_row(ui, &t!("label_power"), &fmt_value(data.gpu_power, 1, "W"));
                info_row(ui, &t!("label_clock"), &fmt_value(data.gpu_clock, 0, "MHz"));
            });

            panel.card(ui, &t!("card_fans"), CardWidth::Half, |ui| {
                fan_gauges(ui, data);
            });

            panel.card(ui, &t!("card_memory"), CardWidth::Half, |ui| {
                info_row(
                    ui,
                    &t!("label_total_memory"),
                    &fmt_value(data.mem_total, 1, "GB"),
                );
                info_row(
                    ui,
                    &t!("label_used_memory"),
                    &fmt_value(data.mem_used, 1, "GB"),
                );
                info_row(
                    ui,
                    &t!("label_remain_memory"),
                    &fmt_value(data.mem_available, 1, "GB"),
                );
                info_row(
                    ui,
                    &t!("label_total_swap"),
                    &fmt_value(data.mem_swap_total, 1, "GB"),
                );
                info_row(
                    ui,
                    &t!("label_used_swap"),
                    &fmt_value(data.mem_swap_used, 1, "GB"),
                );
            });

            panel.card(ui, &t!("card_battery"), CardWidth::Half, |ui| {
                info_row(
                    ui,
                    &t!("label_current_capacity"),
                    &fmt_value(data.bat_capacity_remain, 0, "Wh"),
                );
                info_row(
                    ui,
                    &t!("label_maximum_capacity"),
                    &fmt_value(data.bat_capacity_max, 0, "Wh"),
                );
                info_row(
                    ui,
                    &t!("label_health_percentage"),
                    &fmt_value(bat_health_pct(data), 1, "%"),
                );
                info_row(
                    ui,
                    &t!("label_discharge_rate"),
                    &fmt_value(data.bat_rate, 1, "W"),
                );
                info_row(ui, &t!("label_state"), fmt_text(&data.bat_state));
            });

            // ── System + Disk (half-width pair) ─────────────────────
            panel.card(ui, &t!("card_system"), CardWidth::Half, |ui| {
                info_row(ui, &t!("label_os"), fmt_text(&data.os_name));
                info_row(ui, &t!("label_activation"), fmt_text(&data.os_activated));
                info_row(
                    ui,
                    &t!("label_kernel_version"),
                    fmt_text(&data.os_kernel_version),
                );
                info_row(ui, &t!("label_os_version"), fmt_text(&data.os_version));
                info_row(ui, &t!("label_host_name"), fmt_text(&data.os_host_name));
            });

            // Disk list populated from engine query "disk_disk".
            panel.card(ui, &t!("card_disk"), CardWidth::Half, |ui| {
                if data.disks.is_empty() {
                    info_row(ui, &t!("label_no_disks"), DEFAULT_VALUE);
                } else {
                    let mut disk_idx: usize = 0;
                    let mut removable_idx: usize = 0;
                    for disk in data.disks.iter() {
                        let (prefix, idx) = if disk.is_removable {
                            let i = removable_idx;
                            removable_idx += 1;
                            (t!("label_removable").to_string(), i)
                        } else {
                            let i = disk_idx;
                            disk_idx += 1;
                            (t!("label_disk").to_string(), i)
                        };
                        disk_row(
                            ui,
                            &prefix,
                            idx,
                            &disk.name,
                            &fmt_bytes(disk.total_space),
                            &fmt_bytes(disk.available_space),
                            &format!("{:.1}%", disk.usage_pct()),
                        );
                    }
                }
            });

            // ── Partition (full-width, Windows-only / debug) ────────
            if cfg!(any(target_os = "windows", debug_assertions)) {
                let partitions: &[&str] = &["C:/", "D:/", "E:/"];
                panel.card(ui, &t!("card_partition"), CardWidth::Half, |ui| {
                    for &name in partitions {
                        partition_row(
                            ui,
                            name,
                            DEFAULT_VALUE,
                            DEFAULT_VALUE,
                            DEFAULT_VALUE,
                            DEFAULT_VALUE,
                        );
                    }
                });
            }
        });
    }
}

// ── Row helpers ─────────────────────────────────────────────────

/// A single key–value row inside a card.
fn info_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(value).monospace());
        });
    });
}

/// One row per disk: `Disk 0  Name /dev/nvme0n1p1  Total Size 839.0 GB  Available Size 200.0 GB  62.7%`.
/// Removable disks are labelled `Removable 0` instead of `Disk 0`.
fn disk_row(
    ui: &mut egui::Ui,
    prefix: &str,
    index: usize,
    name: &str,
    total_size: &str,
    available_size: &str,
    usage: &str,
) {
    ui.horizontal(|ui| {
        ui.label(format!("{prefix} {index}"));
        ui.label(
            egui::RichText::new(format!("{name}"))
                .monospace()
                .size(11.0),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(usage).monospace());
            ui.label(
                egui::RichText::new(t!("label_available", size = available_size).to_string())
                    .monospace(),
            );
            ui.label(
                egui::RichText::new(t!("label_total", size = total_size).to_string()).monospace(),
            );
        });
    });
}

/// One row per partition: `C:/   ──GB   ──GB   ──%   BitLocker: ──`.
fn partition_row(
    ui: &mut egui::Ui,
    name: &str,
    size: &str,
    remain: &str,
    pct: &str,
    bitlocker: &str,
) {
    ui.horizontal(|ui| {
        ui.label(name);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // right-to-left: last added appears leftmost
            ui.label(
                egui::RichText::new(t!("label_bitlocker", status = bitlocker).to_string())
                    .monospace(),
            );
            ui.label(egui::RichText::new(pct).monospace());
            ui.label(egui::RichText::new(remain).monospace());
            ui.label(egui::RichText::new(size).monospace());
        });
    });
}

// ── Fan gauges ──────────────────────────────────────────────────

/// Draw three fan speed gauges (CPU, Mid, GPU) side by side.
fn fan_gauges(ui: &mut egui::Ui, data: &SensorData) {
    /// Assumed maximum RPM for computing the gauge fill fraction.
    const MAX_RPM: f32 = 9000.0;

    let fan_cpu_label = t!("fan_cpu");
    let fan_mid_label = t!("fan_mid");
    let fan_gpu_label = t!("fan_gpu");
    let fans: [(&str, Option<i32>); 3] = [
        (&fan_cpu_label, data.fan_cpu),
        (&fan_mid_label, data.fan_mid),
        (&fan_gpu_label, data.fan_gpu),
    ];
    let available = ui.available_size();
    let col_width = available.x / fans.len() as f32;
    let radius = (col_width * 0.35).min(available.y * 0.38);

    ui.horizontal(|ui| {
        for &(label, rpm) in &fans {
            let rpm_text = rpm.map_or_else(|| DEFAULT_VALUE.to_owned(), |v| v.to_string());
            let progress = rpm.map_or(0.0, |v| (v as f32 / MAX_RPM).clamp(0.0, 1.0));

            let (rect, _) =
                ui.allocate_exact_size(egui::vec2(col_width, available.y), egui::Sense::hover());
            draw_fan_gauge(ui, rect, label, &rpm_text, progress, radius);
        }
    });
}

/// Draw a single fan arc gauge with a gap at the top for the label.
fn draw_fan_gauge(
    ui: &egui::Ui,
    rect: egui::Rect,
    label: &str,
    rpm_text: &str,
    progress: f32,
    radius: f32,
) {
    use std::f32::consts::PI;

    let painter = ui.painter();
    // Shift center down slightly so the label in the gap has room.
    let center = egui::pos2(rect.center().x, rect.center().y + 6.0);

    // Arc with a 90° gap at the 12-o'clock position.
    let gap = PI / 2.0;
    let arc_start = 3.0 * PI / 2.0 + gap / 2.0; // upper-right (≈ 1:30)
    let arc_sweep = 2.0 * PI - gap; // 270° clockwise through bottom

    let stroke_w = 5.0;
    let bg_color = ui
        .visuals()
        .widgets
        .noninteractive
        .bg_stroke
        .color
        .gamma_multiply(0.35);
    let fg_color = ui.visuals().selection.bg_fill;

    // Background arc (full track)
    painter.add(egui::Shape::line(
        arc_points(center, radius, arc_start, arc_sweep, 60),
        egui::Stroke::new(stroke_w, bg_color),
    ));

    // Progress arc (filled portion)
    if progress > 0.001 {
        let sweep = arc_sweep * progress.clamp(0.0, 1.0);
        painter.add(egui::Shape::line(
            arc_points(center, radius, arc_start, sweep, 60),
            egui::Stroke::new(stroke_w, fg_color),
        ));
    }

    // Label sitting in the gap at the top of the arc.
    painter.text(
        egui::pos2(center.x, center.y - radius - stroke_w / 2.0 - 2.0),
        egui::Align2::CENTER_BOTTOM,
        label,
        egui::FontId::proportional(11.0),
        ui.visuals().text_color(),
    );

    // RPM value in the center of the arc.
    painter.text(
        egui::pos2(center.x, center.y - 4.0),
        egui::Align2::CENTER_CENTER,
        rpm_text,
        egui::FontId::monospace(12.0),
        ui.visuals().text_color(),
    );

    // "RPM" unit label just below the value.
    painter.text(
        egui::pos2(center.x, center.y + 10.0),
        egui::Align2::CENTER_CENTER,
        &t!("label_rpm"),
        egui::FontId::proportional(9.0),
        ui.visuals().weak_text_color(),
    );
}

/// Generate evenly-spaced points along a circular arc.
fn arc_points(
    center: egui::Pos2,
    radius: f32,
    start: f32,
    sweep: f32,
    segments: usize,
) -> Vec<egui::Pos2> {
    (0..=segments)
        .map(|i| {
            let angle = start + sweep * (i as f32 / segments as f32);
            egui::pos2(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            )
        })
        .collect()
}
