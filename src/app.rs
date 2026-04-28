use crate::sensor_data::{fmt_value, SensorData};
use crate::views::{ActionsView, DashboardView, SettingsView, ToolsView};
use rust_i18n::t;

/// Which tab is currently selected in the sidebar.
#[derive(Default, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum NavTab {
    #[default]
    Dashboard,
    Actions,
    Tools,
    Settings,
}

impl NavTab {
    pub const ALL: [Self; 4] = [Self::Dashboard, Self::Actions, Self::Tools, Self::Settings];

    pub fn label(self) -> String {
        match self {
            Self::Dashboard => t!("tab_dashboard").to_string(),
            Self::Actions => t!("tab_actions").to_string(),
            Self::Tools => t!("tab_tools").to_string(),
            Self::Settings => t!("tab_settings").to_string(),
        }
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    active_tab: NavTab,

    dashboard: DashboardView,
    actions: ActionsView,
    tools: ToolsView,
    settings: SettingsView,

    /// Live sensor readings, refreshed once per frame.
    #[serde(skip)]
    sensor_data: SensorData,

    /// Instant when the app was started, used for uptime display.
    #[serde(skip, default = "std::time::Instant::now")]
    start_time: std::time::Instant,
}

impl Default for App {
    fn default() -> Self {
        Self {
            active_tab: NavTab::default(),
            dashboard: DashboardView::default(),
            actions: ActionsView::default(),
            tools: ToolsView::default(),
            settings: SettingsView::default(),
            sensor_data: SensorData::default(),
            start_time: std::time::Instant::now(),
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load previous app state (if any).
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Self::default()
        }
    }
}

/// Fixed width of the sidebar in logical pixels.
const SIDEBAR_WIDTH: f32 = 180.0;

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // ── Refresh sensor data once per frame ────────────────────
        self.sensor_data.refresh();
        self.tools.update_monitor_plot(&self.sensor_data);

        // ── Left sidebar ───────────────────────────────────────────
        egui::Panel::left("nav_sidebar")
            .exact_size(SIDEBAR_WIDTH)
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.add_space(12.0);
                ui.vertical_centered(|ui| {
                    ui.heading(t!("app_title").to_string());
                });
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                for tab in NavTab::ALL {
                    let selected = self.active_tab == tab;
                    let button = egui::Button::new(egui::RichText::new(tab.label()).size(15.0))
                        .fill(if selected {
                            ui.visuals().selection.bg_fill
                        } else {
                            egui::Color32::TRANSPARENT
                        })
                        .min_size(egui::vec2(ui.available_width(), 36.0));

                    if ui.add(button).clicked() {
                        self.active_tab = tab;
                    }
                    ui.add_space(4.0);
                }

                // ── Hardware monitor + clock (bottom) ────────────
                let data = &self.sensor_data;
                let start_time = self.start_time;
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    // Request repaint so the clock updates every second.
                    ui.ctx().request_repaint();

                    ui.add_space(6.0);

                    // ── Time info (very bottom) ──────────────────
                    let now = chrono::Local::now();
                    let elapsed = start_time.elapsed();
                    let hours = elapsed.as_secs() / 3600;
                    let mins = (elapsed.as_secs() % 3600) / 60;
                    let secs = elapsed.as_secs() % 60;

                    let mono_small = egui::FontId::monospace(11.0);
                    ui.label(
                        egui::RichText::new(format!(
                            "{}  {hours:02}:{mins:02}:{secs:02}",
                            t!("uptime_prefix")
                        ))
                        .font(mono_small.clone())
                        .weak(),
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "{}  {}",
                            t!("time_prefix"),
                            now.format("%H:%M:%S")
                        ))
                        .font(mono_small)
                        .weak(),
                    );
                    ui.add_space(4.0);
                    ui.separator();

                    // ── Hardware rows ────────────────────────────
                    hw_row(
                        ui,
                        &t!("hw_bat"),
                        &fmt_value(crate::sensor_data::bat_health_pct(data), 0, "%"),
                        crate::sensor_data::fmt_text(&data.bat_state),
                    );
                    hw_row(
                        ui,
                        &t!("hw_gpu"),
                        &fmt_value(data.gpu_temperature, 0, "°C"),
                        &fmt_value(data.gpu_power, 1, "W"),
                    );
                    hw_row(
                        ui,
                        &t!("hw_cpu"),
                        &fmt_value(data.cpu_temperature, 0, "°C"),
                        &fmt_value(data.cpu_power, 1, "W"),
                    );

                    ui.add_space(2.0);
                    ui.separator();
                });
            });

        // ── Right content area ─────────────────────────────────────
        let sensor_data = self.sensor_data.clone();
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.add_space(8.0);
            match self.active_tab {
                NavTab::Dashboard => self.dashboard.ui(ui, &sensor_data),
                NavTab::Actions => self.actions.ui(ui),
                NavTab::Tools => self.tools.ui(ui),
                NavTab::Settings => self.settings.ui(ui),
            }
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

/// One row inside the sidebar hardware monitor.
///
/// `label`: e.g. "CPU", `col1`/`col2`: the two value columns.
fn hw_row(ui: &mut egui::Ui, label: &str, col1: &str, col2: &str) {
    ui.horizontal(|ui| {
        let mono = egui::FontId::monospace(12.0);
        ui.label(egui::RichText::new(label).font(mono.clone()).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(col2).font(mono.clone()));
            ui.label(egui::RichText::new(col1).font(mono));
        });
    });
}
