/// Width a card occupies within a [`CardPanel`] row.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CardWidth {
    /// Takes the full row width.
    Full,
    /// Takes half the row, leaving room for another half-width card.
    /// The exact split ratio can be adjusted by dragging the divider
    /// between two half-width cards.
    Half,
}

/// Minimum width (logical pixels) a half-width card can be shrunk to.
const MIN_HALF_WIDTH: f32 = 120.0;

/// Width of the invisible drag hit-area between two half-width cards.
const SEPARATOR_HIT_WIDTH: f32 = 12.0;

/// Fixed-layout card panel that organizes cards into rows.
///
/// Cards have a default height set in [`CardPanel::begin`], but individual
/// cards can override it via [`CardPanel::card_with_height`].  Two widths
/// are available:
/// * **Full** – spans the entire row.
/// * **Half** – spans half the row (two cards fit side-by-side).  The split
///   between two adjacent half-width cards can be adjusted by dragging the
///   divider.  Double-click the divider to reset to an even 50/50 split.
///
/// # Usage
/// ```ignore
/// CardPanel::show(ui, 160.0, |panel, ui| {
///     panel.card(ui, "Left",  CardWidth::Half, |ui| { ui.label("A"); });
///     panel.card(ui, "Right", CardWidth::Half, |ui| { ui.label("B"); });
/// });
/// ```
pub struct CardPanel {
    /// Stable id used to key per-row split ratios in egui memory.
    id: egui::Id,
    card_height: f32,
    spacing: f32,
    origin: egui::Pos2,
    available_width: f32,
    cursor_x: f32,
    cursor_y: f32,
    max_y: f32,
    /// Monotonically increasing row counter (reset each frame via `begin`).
    row_index: usize,
    /// Number of half-width cards placed in the *current* row so far.
    row_half_count: usize,
    /// Right edge of the first half-width card in the current row.
    row_first_card_right: f32,
}

impl CardPanel {
    /// Build a scrollable card panel.
    ///
    /// This wraps the panel in an [`egui::ScrollArea`] so it scrolls
    /// vertically when the content exceeds the available height.
    ///
    /// ```ignore
    /// CardPanel::show(ui, 160.0, |panel, ui| {
    ///     panel.card(ui, "Left",  CardWidth::Half, |ui| { ui.label("A"); });
    ///     panel.card(ui, "Right", CardWidth::Half, |ui| { ui.label("B"); });
    /// });
    /// ```
    pub fn show(
        ui: &mut egui::Ui,
        card_height: f32,
        build: impl FnOnce(&mut Self, &mut egui::Ui),
    ) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut panel = Self::begin(ui, card_height);
            build(&mut panel, ui);
            panel.end(ui);
        });
    }

    /// Start building a panel.  Call [`Self::card`] to add cards, then
    /// [`Self::end`] to finalize the layout.
    pub fn begin(ui: &egui::Ui, card_height: f32) -> Self {
        let id = ui.id().with("card_panel");
        let origin = ui.next_widget_position();
        let available_width = ui.available_width();
        Self {
            id,
            card_height,
            spacing: 8.0,
            origin,
            available_width,
            cursor_x: origin.x,
            cursor_y: origin.y,
            max_y: origin.y,
            row_index: 0,
            row_half_count: 0,
            row_first_card_right: 0.0,
        }
    }

    // ── helpers ──────────────────────────────────────────────────

    /// Move the cursor to the start of a new row.
    fn advance_row(&mut self) {
        self.cursor_x = self.origin.x;
        self.cursor_y = self.max_y + self.spacing;
        self.row_index += 1;
        self.row_half_count = 0;
    }

    /// Read the stored split ratio for the current row (default 0.5).
    fn row_ratio(&self, ui: &egui::Ui) -> f32 {
        let row_id = self.id.with(("row_ratio", self.row_index));
        ui.data(|d| d.get_temp::<f32>(row_id)).unwrap_or(0.5)
    }

    /// Persist a new split ratio for the current row.
    fn set_row_ratio(&self, ui: &egui::Ui, ratio: f32) {
        let row_id = self.id.with(("row_ratio", self.row_index));
        ui.data_mut(|d| d.insert_temp(row_id, ratio));
    }

    /// Usable width for a pair of half-width cards (total minus the gap).
    fn usable_width(&self) -> f32 {
        self.available_width - self.spacing
    }

    /// Minimum half-card width, capped so both sides always fit.
    fn min_half(&self) -> f32 {
        MIN_HALF_WIDTH.min(self.usable_width() * 0.2)
    }

    // ── public API ──────────────────────────────────────────────

    /// Add a card with the given `title` and `width`, using the default
    /// panel height set in [`Self::begin`].
    ///
    /// The `content` closure receives a [`egui::Ui`] scoped to the card's
    /// content area (below the title and separator).
    pub fn card(
        &mut self,
        ui: &mut egui::Ui,
        title: &str,
        width: CardWidth,
        content: impl FnOnce(&mut egui::Ui),
    ) {
        let height = self.card_height;
        self.card_impl(ui, title, width, height, content);
    }

    /// Like [`Self::card`] but with an explicit height override.
    ///
    /// Useful for cards whose content is dynamically sized (e.g. a
    /// variable number of rows).
    pub fn card_with_height(
        &mut self,
        ui: &mut egui::Ui,
        title: &str,
        width: CardWidth,
        height: f32,
        content: impl FnOnce(&mut egui::Ui),
    ) {
        self.card_impl(ui, title, width, height, content);
    }

    fn card_impl(
        &mut self,
        ui: &mut egui::Ui,
        title: &str,
        width: CardWidth,
        card_height: f32,
        content: impl FnOnce(&mut egui::Ui),
    ) {
        // ── compute card width ───────────────────────────────────
        let card_width = match width {
            CardWidth::Full => {
                // A full-width card always starts a fresh row.
                if self.row_half_count > 0 {
                    self.advance_row();
                }
                self.available_width
            }
            CardWidth::Half => {
                // If the previous row already had two halves, start a new one.
                if self.row_half_count >= 2 {
                    self.advance_row();
                }
                let usable = self.usable_width();
                let min_w = self.min_half();
                let ratio = self.row_ratio(ui);
                let first_w = (usable * ratio).clamp(min_w, usable - min_w);
                if self.row_half_count == 0 {
                    first_w
                } else {
                    usable - first_w
                }
            }
        };

        // ── place card ───────────────────────────────────────────
        let card_rect = egui::Rect::from_min_size(
            egui::pos2(self.cursor_x, self.cursor_y),
            egui::vec2(card_width, card_height),
        );

        // Draw card background & border
        let rounding = 6.0;
        ui.painter()
            .rect_filled(card_rect, rounding, ui.visuals().faint_bg_color);
        ui.painter().rect_stroke(
            card_rect,
            rounding,
            egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
            egui::StrokeKind::Inside,
        );

        // Inner area with margin
        let inner_rect = card_rect.shrink(12.0);
        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(inner_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );

        // Title
        child_ui.label(egui::RichText::new(title).strong().size(15.0));
        child_ui.add_space(2.0);
        child_ui.separator();
        child_ui.add_space(4.0);

        // Content
        content(&mut child_ui);

        // ── advance cursor ───────────────────────────────────────
        self.cursor_x += card_width + self.spacing;
        self.max_y = self.max_y.max(card_rect.bottom());

        // ── row bookkeeping ──────────────────────────────────────
        match width {
            CardWidth::Full => {
                self.advance_row();
            }
            CardWidth::Half => {
                self.row_half_count += 1;
                if self.row_half_count == 1 {
                    self.row_first_card_right = card_rect.right();
                } else if self.row_half_count == 2 {
                    // Two halves are placed – draw the draggable divider.
                    self.draw_separator(ui, card_rect.left());
                    self.advance_row();
                }
            }
        }
    }

    /// Draw an interactive vertical divider between two half-width cards.
    fn draw_separator(&self, ui: &mut egui::Ui, second_card_left: f32) {
        let gap_center_x = (self.row_first_card_right + second_card_left) / 2.0;
        let top = self.cursor_y;
        let bottom = self.max_y;

        // Invisible hit area covering the gap.
        let hit_rect = egui::Rect::from_x_y_ranges(
            (gap_center_x - SEPARATOR_HIT_WIDTH / 2.0)..=(gap_center_x + SEPARATOR_HIT_WIDTH / 2.0),
            top..=bottom,
        );

        let sep_id = self.id.with(("separator", self.row_index));
        let response = ui.interact(hit_rect, sep_id, egui::Sense::click_and_drag());

        // Cursor feedback.
        if response.hovered() || response.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeColumn);
        }

        // Drag → adjust ratio.
        if response.dragged() {
            let usable = self.usable_width();
            let min_w = self.min_half();
            let ratio = self.row_ratio(ui);
            let cur_first = usable * ratio;
            let new_first = (cur_first + response.drag_delta().x).clamp(min_w, usable - min_w);
            self.set_row_ratio(ui, new_first / usable);
        }

        // Double-click → reset to 50/50.
        if response.double_clicked() {
            self.set_row_ratio(ui, 0.5);
        }

        // Visual hint on hover / drag.
        if response.hovered() || response.dragged() {
            ui.painter().vline(
                gap_center_x,
                top..=bottom,
                egui::Stroke::new(2.0, ui.visuals().selection.bg_fill),
            );
        }
    }

    /// Finish the panel, reserving the used space in the parent [`egui::Ui`].
    pub fn end(self, ui: &mut egui::Ui) {
        let total_rect = egui::Rect::from_min_max(
            self.origin,
            egui::pos2(self.origin.x + self.available_width, self.max_y),
        );
        ui.allocate_rect(total_rect, egui::Sense::hover());
    }
}


