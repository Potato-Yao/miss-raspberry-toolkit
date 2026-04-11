#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct SettingsView;

impl SettingsView {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Settings");
        ui.separator();
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label("Theme:");
            egui::widgets::global_theme_preference_buttons(ui);
        });
    }
}
