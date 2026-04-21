use super::card::{CardPanel, CardWidth};
use rust_i18n::t;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct ToolsView;

const CARD_HEIGHT: f32 = 200.0;

impl ToolsView {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let is_windows_or_test = cfg!(target_os = "windows") || cfg!(debug_assertions);

        CardPanel::show(ui, CARD_HEIGHT, |panel, ui| {
            // ── Stress Test ───────────────────────────────────────
            panel.card(ui, &t!("tools_stress_test"), CardWidth::Full, |ui| {
                ui.horizontal_wrapped(|ui| {
                    let _ = ui.button(t!("tools_auto_judgement").to_string());
                    let _ = ui.button(t!("tools_cpu_test").to_string());
                    let _ = ui.button(t!("tools_gpu_test").to_string());
                    let _ = ui.button(t!("tools_cpu_gpu_test").to_string());
                    let _ = ui.button(t!("tools_ram_test").to_string());
                    let _ = ui.button(t!("tools_monitor_plot").to_string());
                });
            });

            // ── System Settings ───────────────────────────────────
            panel.card(ui, &t!("tools_system_settings"), CardWidth::Full, |ui| {
                ui.horizontal_wrapped(|ui| {
                    let _ = ui.button(t!("tools_boot_manager").to_string());
                    if is_windows_or_test {
                        let _ = ui.button(t!("tools_bitlocker_manager").to_string());
                        let _ = ui.button(t!("tools_activator").to_string());
                    }
                });
            });

            // ── Fix ───────────────────────────────────────────────
            panel.card(ui, &t!("tools_fix"), CardWidth::Full, |ui| {
                ui.horizontal_wrapped(|ui| {
                    let _ = ui.button(t!("tools_network_issue_fix").to_string());
                });
            });

            // ── Others ────────────────────────────────────────────
            panel.card(ui, &t!("tools_others"), CardWidth::Full, |ui| {
                ui.horizontal_wrapped(|ui| {
                    let _ = ui.button(t!("tools_restart_to_bios").to_string());
                });
            });
        });
    }
}
