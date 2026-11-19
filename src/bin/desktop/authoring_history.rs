//! Contents shortcut routing; timeline history and force selection retain their owners.
use eframe::egui;
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Action {
    Cancel,
    Undo,
    Redo,
}
pub(super) fn action(ui: &egui::Ui, selected_draft: bool) -> Option<Action> {
    let text_focused = ui
        .ctx()
        .memory(|m| m.focused())
        .is_some_and(|id| egui::TextEdit::load_state(ui.ctx(), id).is_some());
    if !selected_draft || text_focused {
        return None;
    }
    ui.input_mut(|i| {
        if i.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
            Some(Action::Cancel)
        } else if i.consume_key(
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
            egui::Key::Z,
        ) {
            Some(Action::Redo)
        } else if i.consume_key(egui::Modifiers::COMMAND, egui::Key::Z) {
            Some(Action::Undo)
        } else {
            None
        }
    })
}
