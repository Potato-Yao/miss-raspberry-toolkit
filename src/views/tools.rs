use super::card::{CardPanel, CardWidth};
use crate::sensor_data::SensorData;
use multimeter_engine::external_program::stress_test_manager::{StressTestManager, TestKind};
use rust_i18n::t;
use std::time::{Duration, Instant};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ToolsView {
    #[serde(skip)]
    stress_tests: StressTestManager,
    #[serde(skip)]
    selected_stress_test: Option<TestKind>,
    #[serde(skip)]
    cpu_test: TestRunState,
    #[serde(skip)]
    gpu_test: TestRunState,
    #[serde(skip)]
    ram_test: TestRunState,
    #[serde(skip)]
    cpu_gpu_test: TestRunState,
    show_monitor_plot: bool,
    #[serde(skip)]
    monitor_plot: MonitorPlotState,
}

impl Default for ToolsView {
    fn default() -> Self {
        Self {
            stress_tests: StressTestManager::new(),
            selected_stress_test: None,
            cpu_test: TestRunState::default(),
            gpu_test: TestRunState::default(),
            ram_test: TestRunState::default(),
            cpu_gpu_test: TestRunState::default(),
            show_monitor_plot: false,
            monitor_plot: MonitorPlotState::default(),
        }
    }
}

#[derive(Default)]
struct TestRunState {
    started_at: Option<Instant>,
    elapsed_before_start: Duration,
    status: Option<String>,
}

impl TestRunState {
    fn is_running(&self) -> bool {
        self.started_at.is_some()
    }

    fn elapsed(&self) -> Duration {
        self.elapsed_before_start
            + self
                .started_at
                .map(|started_at| started_at.elapsed())
                .unwrap_or_default()
    }

    fn mark_started(&mut self) {
        self.started_at = Some(Instant::now());
        self.elapsed_before_start = Duration::ZERO;
        self.status = Some(t!("tools_test_running").to_string());
    }

    fn mark_stopped(&mut self) {
        self.elapsed_before_start = self.elapsed();
        self.started_at = None;
        self.status = Some(t!("tools_test_stopped").to_string());
    }

    fn mark_failed(&mut self, action: &str, error: String) {
        self.status = Some(format!("{action}: {error}"));
    }
}

const CARD_HEIGHT: f32 = 200.0;
const MONITOR_PLOT_HEIGHT: f32 = 520.0;
const MONITOR_PLOT_CANVAS_HEIGHT: f32 = 185.0;
const MONITOR_PLOT_DEFAULT_TIME_SECONDS: f64 = 60.0;
const MONITOR_PLOT_MAX_TIME_SECONDS: f64 = 300.0;
const MONITOR_PLOT_TEMPERATURE_C: f64 = 80.0;
const MONITOR_PLOT_POWER_W: f64 = 40.0;
const MONITOR_PLOT_SAMPLE_INTERVAL: Duration = Duration::from_millis(500);

impl ToolsView {
    pub fn update_monitor_plot(&mut self, sensor_data: &SensorData) {
        self.monitor_plot.sample(sensor_data);
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let is_windows_or_test = cfg!(target_os = "windows") || cfg!(debug_assertions);

        CardPanel::show(ui, CARD_HEIGHT, |panel, ui| {
            // ── Stress Test ───────────────────────────────────────
            panel.card(ui, &t!("tools_stress_test"), CardWidth::Full, |ui| {
                ui.horizontal_wrapped(|ui| {
                    let _ = ui.button(t!("tools_auto_judgement").to_string());
                    if ui
                        .add_enabled(
                            self.stress_tests.is_available(TestKind::Cpu),
                            egui::Button::new(t!("tools_cpu_test").to_string()),
                        )
                        .clicked()
                    {
                        self.toggle_stress_test(TestKind::Cpu);
                    }
                    if ui
                        .add_enabled(
                            self.stress_tests.is_available(TestKind::Gpu),
                            egui::Button::new(t!("tools_gpu_test").to_string()),
                        )
                        .clicked()
                    {
                        self.toggle_stress_test(TestKind::Gpu);
                    }
                    if ui
                        .add_enabled(
                            self.stress_tests.is_available(TestKind::CpuGpu),
                            egui::Button::new(t!("tools_cpu_gpu_test").to_string()),
                        )
                        .clicked()
                    {
                        self.toggle_stress_test(TestKind::CpuGpu);
                    }
                    if ui
                        .add_enabled(
                            self.stress_tests.is_available(TestKind::Ram),
                            egui::Button::new(t!("tools_ram_test").to_string()),
                        )
                        .clicked()
                    {
                        self.toggle_stress_test(TestKind::Ram);
                    }
                    if ui.button(t!("tools_monitor_plot").to_string()).clicked() {
                        self.show_monitor_plot = !self.show_monitor_plot;
                    }
                });
            });

            if self.show_monitor_plot {
                panel.card_with_height(
                    ui,
                    &t!("tools_monitor_plot"),
                    CardWidth::Full,
                    MONITOR_PLOT_HEIGHT,
                    |ui| {
                        self.monitor_plot_control(ui);
                    },
                );
            }

            if let Some(kind) = self.selected_stress_test {
                let title = format!("{} {}", stress_test_label(kind), t!("tools_test_control"));

                panel.card(ui, &title, CardWidth::Full, |ui| {
                    self.stress_test_control(ui, kind);
                });
            }

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

    fn toggle_stress_test(&mut self, kind: TestKind) {
        self.selected_stress_test = if self.selected_stress_test == Some(kind) {
            None
        } else {
            Some(kind)
        };
    }

    fn stress_test_control(&mut self, ui: &mut egui::Ui, kind: TestKind) {
        let available = self.stress_tests.is_available(kind);
        let is_running = self.test_state(kind).is_running();

        if is_running {
            ui.ctx().request_repaint_after(Duration::from_secs(1));
        }

        ui.horizontal(|ui| {
            ui.label(t!("tools_test_elapsed").to_string());
            ui.label(
                egui::RichText::new(format_duration(self.test_state(kind).elapsed()))
                    .monospace()
                    .strong(),
            );
        });

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    available && !is_running,
                    egui::Button::new(t!("tools_test_start").to_string()),
                )
                .clicked()
            {
                match self.stress_tests.start(kind) {
                    Ok(()) => self.test_state_mut(kind).mark_started(),
                    Err(error) => self
                        .test_state_mut(kind)
                        .mark_failed(&t!("tools_test_start_failed"), error.to_string()),
                }
            }

            if ui
                .add_enabled(
                    available && is_running,
                    egui::Button::new(t!("tools_test_stop").to_string()),
                )
                .clicked()
            {
                match self.stress_tests.close(kind) {
                    Ok(()) => self.test_state_mut(kind).mark_stopped(),
                    Err(error) => {
                        self.test_state_mut(kind).mark_stopped();
                        self.test_state_mut(kind)
                            .mark_failed(&t!("tools_test_stop_failed"), error.to_string());
                    }
                }
            }
        });

        if !available {
            ui.add_space(6.0);
            ui.colored_label(
                ui.visuals().warn_fg_color,
                t!("tools_test_unavailable").to_string(),
            );
        } else if let Some(status) = &self.test_state(kind).status {
            ui.add_space(6.0);
            ui.label(egui::RichText::new(status).weak());
        }
    }

    fn monitor_plot_control(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("reset").clicked() {
                self.monitor_plot.reset();
            }

            if ui
                .add_enabled(!self.monitor_plot.is_running(), egui::Button::new("start"))
                .clicked()
            {
                self.monitor_plot.start();
            }

            if ui
                .add_enabled(self.monitor_plot.is_running(), egui::Button::new("pause"))
                .clicked()
            {
                self.monitor_plot.pause();
            }
        });

        if self.monitor_plot.is_running() {
            ui.ctx().request_repaint_after(MONITOR_PLOT_SAMPLE_INTERVAL);
        }

        ui.add_space(8.0);
        draw_monitor_plot(ui, &self.monitor_plot.samples);
    }

    fn test_state(&self, kind: TestKind) -> &TestRunState {
        match kind {
            TestKind::Cpu => &self.cpu_test,
            TestKind::Gpu => &self.gpu_test,
            TestKind::Ram => &self.ram_test,
            TestKind::CpuGpu => &self.cpu_gpu_test,
        }
    }

    fn test_state_mut(&mut self, kind: TestKind) -> &mut TestRunState {
        match kind {
            TestKind::Cpu => &mut self.cpu_test,
            TestKind::Gpu => &mut self.gpu_test,
            TestKind::Ram => &mut self.ram_test,
            TestKind::CpuGpu => &mut self.cpu_gpu_test,
        }
    }
}

#[derive(Default)]
struct MonitorPlotState {
    started_at: Option<Instant>,
    elapsed_before_start: Duration,
    samples: Vec<MonitorPlotSample>,
    last_sample_at: Option<Instant>,
}

impl MonitorPlotState {
    fn is_running(&self) -> bool {
        self.started_at.is_some()
    }

    fn elapsed(&self) -> Duration {
        self.elapsed_before_start
            + self
                .started_at
                .map(|started_at| started_at.elapsed())
                .unwrap_or_default()
    }

    fn start(&mut self) {
        if self.is_running() {
            return;
        }

        self.started_at = Some(Instant::now());
        self.last_sample_at = None;
    }

    fn pause(&mut self) {
        if !self.is_running() {
            return;
        }

        self.elapsed_before_start = self.elapsed();
        self.started_at = None;
        self.last_sample_at = None;
    }

    fn reset(&mut self) {
        self.started_at = None;
        self.elapsed_before_start = Duration::ZERO;
        self.samples.clear();
        self.last_sample_at = None;
    }

    fn sample(&mut self, sensor_data: &SensorData) {
        if !self.is_running() {
            return;
        }

        let now = Instant::now();
        if self.last_sample_at.is_some_and(|last_sample_at| {
            now.duration_since(last_sample_at) < MONITOR_PLOT_SAMPLE_INTERVAL
        }) {
            return;
        }

        self.last_sample_at = Some(now);

        let cpu_temperature_c = valid_sensor_value(sensor_data.cpu_temperature);
        let gpu_temperature_c = valid_sensor_value(sensor_data.gpu_temperature);
        let cpu_power_w = valid_sensor_value(sensor_data.cpu_power);
        let gpu_power_w = valid_sensor_value(sensor_data.gpu_power);

        if cpu_temperature_c.is_none()
            && gpu_temperature_c.is_none()
            && cpu_power_w.is_none()
            && gpu_power_w.is_none()
        {
            return;
        };

        self.samples.push(MonitorPlotSample {
            time_seconds: self.elapsed().as_secs_f64(),
            cpu_temperature_c,
            gpu_temperature_c,
            cpu_power_w,
            gpu_power_w,
        });
    }
}

struct MonitorPlotSample {
    time_seconds: f64,
    cpu_temperature_c: Option<f64>,
    gpu_temperature_c: Option<f64>,
    cpu_power_w: Option<f64>,
    gpu_power_w: Option<f64>,
}

fn valid_sensor_value(value: Option<f64>) -> Option<f64> {
    value.filter(|value| value.is_finite())
}

fn draw_monitor_plot(ui: &mut egui::Ui, samples: &[MonitorPlotSample]) {
    ui.label(egui::RichText::new("Temperature").strong());
    draw_metric_plot(
        ui,
        samples,
        MONITOR_PLOT_TEMPERATURE_C,
        "°C",
        |sample| sample.cpu_temperature_c,
        |sample| sample.gpu_temperature_c,
    );

    ui.add_space(10.0);
    ui.label(egui::RichText::new("Power").strong());
    draw_metric_plot(
        ui,
        samples,
        MONITOR_PLOT_POWER_W,
        "W",
        |sample| sample.cpu_power_w,
        |sample| sample.gpu_power_w,
    );
}

fn draw_metric_plot(
    ui: &mut egui::Ui,
    samples: &[MonitorPlotSample],
    default_max_value: f64,
    unit: &str,
    cpu_value: impl Fn(&MonitorPlotSample) -> Option<f64> + Copy,
    gpu_value: impl Fn(&MonitorPlotSample) -> Option<f64> + Copy,
) {
    let desired_size = egui::vec2(ui.available_width(), MONITOR_PLOT_CANVAS_HEIGHT);
    let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
    let painter = ui.painter_at(rect);

    let plot_rect = egui::Rect::from_min_max(
        egui::pos2(rect.left() + 42.0, rect.top() + 8.0),
        egui::pos2(rect.right() - 10.0, rect.bottom() - 28.0),
    );

    let axis_color = ui.visuals().widgets.noninteractive.fg_stroke.color;
    let grid_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
    let cpu_color = ui.visuals().selection.bg_fill;
    let gpu_color = ui.visuals().warn_fg_color;
    let text_color = ui.visuals().weak_text_color();

    painter.rect_stroke(
        plot_rect,
        0.0,
        egui::Stroke::new(1.0, axis_color),
        egui::StrokeKind::Inside,
    );

    let latest_time = samples
        .last()
        .map(|sample| sample.time_seconds)
        .unwrap_or_default();
    let (visible_start_time, visible_end_time, visible_duration) = visible_time_window(latest_time);
    let max_observed_value = samples
        .iter()
        .filter(|sample| {
            sample.time_seconds >= visible_start_time && sample.time_seconds <= visible_end_time
        })
        .flat_map(|sample| [cpu_value(sample), gpu_value(sample)])
        .flatten()
        .fold(None, |max_value: Option<f64>, value| {
            Some(max_value.map_or(value, |max| max.max(value)))
        });
    let max_value = if let Some(max_observed_value) = max_observed_value {
        (max_observed_value * 1.2).max(1.0)
    } else {
        default_max_value
    };

    for step in 0..=3 {
        let t = step as f32 / 3.0;
        let x = egui::lerp(plot_rect.left()..=plot_rect.right(), t);
        painter.vline(
            x,
            plot_rect.top()..=plot_rect.bottom(),
            egui::Stroke::new(1.0, grid_color),
        );

        painter.text(
            egui::pos2(x, plot_rect.bottom() + 6.0),
            egui::Align2::CENTER_TOP,
            format!("{:.0}s", visible_start_time + visible_duration * t as f64),
            egui::FontId::monospace(11.0),
            text_color,
        );
    }

    for step in 0..=8 {
        let t = step as f32 / 8.0;
        let y = egui::lerp(plot_rect.bottom()..=plot_rect.top(), t);

        painter.hline(
            plot_rect.left()..=plot_rect.right(),
            y,
            egui::Stroke::new(1.0, grid_color),
        );

        painter.text(
            egui::pos2(plot_rect.left() - 8.0, y),
            egui::Align2::RIGHT_CENTER,
            format_metric_axis_label(max_value * t as f64),
            egui::FontId::monospace(11.0),
            text_color,
        );
    }

    painter.text(
        egui::pos2(plot_rect.left() - 8.0, plot_rect.top() - 2.0),
        egui::Align2::RIGHT_BOTTOM,
        unit,
        egui::FontId::monospace(11.0),
        text_color,
    );

    if samples.is_empty() {
        return;
    }

    draw_metric_series(
        &painter,
        samples,
        plot_rect,
        visible_start_time,
        visible_duration,
        max_value,
        cpu_value,
        cpu_color,
    );
    draw_metric_series(
        &painter,
        samples,
        plot_rect,
        visible_start_time,
        visible_duration,
        max_value,
        gpu_value,
        gpu_color,
    );

    draw_plot_legend(&painter, plot_rect, cpu_color, gpu_color, text_color);
}

fn visible_time_window(latest_time: f64) -> (f64, f64, f64) {
    let start_time = (latest_time - MONITOR_PLOT_MAX_TIME_SECONDS).max(0.0);
    let end_time = if latest_time > MONITOR_PLOT_MAX_TIME_SECONDS {
        latest_time
    } else {
        latest_time.max(MONITOR_PLOT_DEFAULT_TIME_SECONDS)
    };

    (start_time, end_time, end_time - start_time)
}

fn format_metric_axis_label(value: f64) -> String {
    if value < 20.0 {
        format!("{value:.1}")
    } else {
        format!("{value:.0}")
    }
}

fn draw_metric_series(
    painter: &egui::Painter,
    samples: &[MonitorPlotSample],
    plot_rect: egui::Rect,
    visible_start_time: f64,
    visible_duration: f64,
    max_value: f64,
    value: impl Fn(&MonitorPlotSample) -> Option<f64>,
    color: egui::Color32,
) {
    let points: Vec<egui::Pos2> = samples
        .iter()
        .filter_map(|sample| {
            if sample.time_seconds < visible_start_time
                || sample.time_seconds > visible_start_time + visible_duration
            {
                return None;
            }

            let metric_value = value(sample)?;
            let x_ratio = ((sample.time_seconds - visible_start_time) / visible_duration)
                .clamp(0.0, 1.0) as f32;
            let y_ratio = (metric_value / max_value).clamp(0.0, 1.0) as f32;
            Some(egui::pos2(
                egui::lerp(plot_rect.left()..=plot_rect.right(), x_ratio),
                egui::lerp(plot_rect.bottom()..=plot_rect.top(), y_ratio),
            ))
        })
        .collect();

    if points.len() >= 2 {
        painter.add(egui::Shape::line(
            points.clone(),
            egui::Stroke::new(2.0, color),
        ));
    }

    for point in points {
        painter.circle_filled(point, 2.5, color);
    }
}

fn draw_plot_legend(
    painter: &egui::Painter,
    plot_rect: egui::Rect,
    cpu_color: egui::Color32,
    gpu_color: egui::Color32,
    text_color: egui::Color32,
) {
    let font = egui::FontId::monospace(11.0);
    let y = plot_rect.top() + 8.0;
    let cpu_x = plot_rect.left() + 10.0;
    let gpu_x = cpu_x + 58.0;

    painter.hline(cpu_x..=cpu_x + 18.0, y, egui::Stroke::new(2.0, cpu_color));
    painter.text(
        egui::pos2(cpu_x + 24.0, y),
        egui::Align2::LEFT_CENTER,
        "CPU",
        font.clone(),
        text_color,
    );

    painter.hline(gpu_x..=gpu_x + 18.0, y, egui::Stroke::new(2.0, gpu_color));
    painter.text(
        egui::pos2(gpu_x + 24.0, y),
        egui::Align2::LEFT_CENTER,
        "GPU",
        font,
        text_color,
    );
}

fn stress_test_label(kind: TestKind) -> String {
    match kind {
        TestKind::Cpu => t!("tools_cpu_test").to_string(),
        TestKind::Gpu => t!("tools_gpu_test").to_string(),
        TestKind::Ram => t!("tools_ram_test").to_string(),
        TestKind::CpuGpu => t!("tools_cpu_gpu_test").to_string(),
    }
}

fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    format!("{hours:02}:{minutes:02}:{seconds:02}")
}
