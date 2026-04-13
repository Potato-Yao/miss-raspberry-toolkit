use crate::sensor_data::{SensorData, fmt_value};
use crate::views::{ActionsView, DashboardView, SettingsView, ToolsView};

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
    pub const ALL: [Self; 4] = [
        Self::Dashboard,
        Self::Actions,
        Self::Tools,
        Self::Settings,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Actions => "Actions",
            Self::Tools => "Tools",
            Self::Settings => "Settings",
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

        // ── Left sidebar ───────────────────────────────────────────
        egui::Panel::left("nav_sidebar")
            .exact_size(SIDEBAR_WIDTH)
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.add_space(12.0);
                ui.vertical_centered(|ui| {
                    ui.heading("Miss Raspberry Toolkit");
                });
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                for tab in NavTab::ALL {
                    let selected = self.active_tab == tab;
                    let button = egui::Button::new(
                        egui::RichText::new(tab.label()).size(15.0),
                    )
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

                // ── Hardware monitor (bottom) ──────────────────────
                let data = &self.sensor_data;
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.add_space(6.0);

                    // Rows are added bottom-to-top in this layout,
                    // so BAT first, then GPU, then CPU, then separator.
                    hw_row(
                        ui,
                        "BAT",
                        &fmt_value(
                            crate::sensor_data::bat_health_pct(data),
                            0,
                            "%",
                        ),
                        &fmt_value(data.bat_rate, 1, "W"),
                    );
                    hw_row(
                        ui,
                        "GPU",
                        &fmt_value(data.gpu_temperature, 0, "°C"),
                        &fmt_value(data.gpu_power, 1, "W"),
                    );
                    hw_row(
                        ui,
                        "CPU",
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
