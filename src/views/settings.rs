use rust_i18n::t;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct SettingsView;

impl SettingsView {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading(t!("settings_heading").to_string());
        ui.separator();
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label(t!("settings_theme").to_string());
            egui::widgets::global_theme_preference_buttons(ui);
        });

        ui.horizontal(|ui| {
            ui.label(t!("settings_language").to_string());
            let current = rust_i18n::locale();
            let mut selected = current.to_string();
            egui::ComboBox::from_id_salt("language_selector")
                .selected_text(&selected)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_value(&mut selected, "en".to_string(), "English")
                        .changed()
                    {
                        rust_i18n::set_locale(&selected);
                    }
                });
        });
    }
}
