use rust_i18n::t;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct ToolsView;

impl ToolsView {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading(t!("tools_heading").to_string());
        ui.separator();
        ui.label(t!("tools_placeholder").to_string());
    }
}
