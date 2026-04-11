#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct ToolsView;

impl ToolsView {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Tools");
        ui.separator();
        ui.label("Tools will appear here.");
    }
}

