use super::card::{CardPanel, CardWidth};
use multimeter_engine::external_program::program::{ExternalProgram, ProgramKind};
use rust_i18n::t;
use std::path::Path;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct ActionsView {
    /// Cached list of tool relative paths.
    #[serde(skip)]
    tools: Option<Vec<String>>,
}

/// Fixed card height used on the actions page.
const CARD_HEIGHT: f32 = 200.0;

impl ActionsView {
    fn ensure_tools(&mut self) -> &[String] {
        if self.tools.is_none() {
            self.tools = Some(ExternalProgram::get_tools().unwrap_or_default());
        }
        self.tools.as_deref().unwrap_or_default()
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        self.ensure_tools();
        let tools = self.tools.clone().unwrap_or_default();

        CardPanel::show(ui, CARD_HEIGHT, |panel, ui| {
            panel.card(ui, &t!("actions_card_title"), CardWidth::Full, |ui| {
                if tools.is_empty() {
                    ui.label(t!("actions_no_tools").to_string());
                } else {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            for tool_path in &tools {
                                let file_name = Path::new(tool_path)
                                    .file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| tool_path.clone());

                                if ui.button(&file_name).clicked() {
                                    let mut program = ExternalProgram::new_transient(
                                        tool_path.clone(),
                                        ProgramKind::Executable,
                                        vec![vec![""]],
                                    );
                                    let _ = program.start(0);
                                }
                            }
                        });
                    });
                }
            });
        });
    }
}
