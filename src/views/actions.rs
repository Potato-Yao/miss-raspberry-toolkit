#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct ActionsView;

impl ActionsView {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Actions");
        ui.separator();
        ui.label("Actions will appear here.");
    }
}

