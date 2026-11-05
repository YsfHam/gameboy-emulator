use eframe::egui::{self, Color32, FontFamily, FontId, RichText, Stroke};

pub const PANEL_BG: Color32 = Color32::from_rgb(25, 29, 31);
pub const CARD_BG: Color32 = Color32::from_rgb(32, 38, 40);
pub const CARD_BORDER: Color32 = Color32::from_rgb(52, 62, 64);
pub const TEXT: Color32 = Color32::from_rgb(225, 232, 224);
pub const MUTED_TEXT: Color32 = Color32::from_rgb(139, 153, 145);
pub const ACCENT: Color32 = Color32::from_rgb(164, 205, 91);
pub const ACCENT_DIM: Color32 = Color32::from_rgb(54, 72, 43);
pub const ACTIVE_BG: Color32 = Color32::from_rgb(43, 58, 39);

pub fn configure_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = PANEL_BG;
    visuals.window_fill = PANEL_BG;
    visuals.faint_bg_color = CARD_BG;
    visuals.extreme_bg_color = Color32::from_rgb(17, 20, 21);
    visuals.override_text_color = Some(TEXT);
    visuals.selection.bg_fill = ACCENT_DIM;
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.noninteractive.bg_fill = CARD_BG;
    visuals.widgets.noninteractive.fg_stroke.color = TEXT;
    visuals.widgets.inactive.bg_fill = CARD_BG;
    visuals.widgets.inactive.fg_stroke.color = MUTED_TEXT;
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(44, 53, 48);
    visuals.widgets.hovered.fg_stroke.color = ACCENT;
    visuals.widgets.active.bg_fill = ACCENT_DIM;
    visuals.widgets.active.fg_stroke.color = ACCENT;
    visuals.collapsing_header_frame = false;
    ctx.set_visuals(visuals);

    ctx.style_mut_of(egui::Theme::Dark, |style| {
        style.spacing.item_spacing = egui::vec2(8.0, 7.0);
        style.text_styles.insert(
            egui::TextStyle::Body,
            FontId::new(13.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            FontId::new(13.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Small,
            FontId::new(11.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Heading,
            FontId::new(16.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Monospace,
            FontId::new(12.0, FontFamily::Monospace),
        );
    });
}

pub fn component_section(ui: &mut egui::Ui, name: &str, body: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(CARD_BG)
        .stroke(Stroke::new(1.0, CARD_BORDER))
        .corner_radius(6.0)
        .inner_margin(8.0)
        .show(ui, |ui| {
            let id = ui.make_persistent_id(("debugger_component", name));
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
                .show_header(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.label(RichText::new(name).size(12.0).strong().color(ACCENT));
                    });
                })
                .body(|ui| {
                    ui.add_space(4.0);
                    body(ui);
                });
        });
    ui.add_space(8.0);
}

pub fn register_cell(ui: &mut egui::Ui, name: &str, value: u16) {
    ui.label(RichText::new(name).small().strong().color(MUTED_TEXT));
    ui.label(
        RichText::new(format!("{value:04X}"))
            .monospace()
            .color(ACCENT),
    );
}

pub fn state_chip(ui: &mut egui::Ui, name: &str, active: bool) {
    let (background, foreground) = if active {
        (ACCENT_DIM, ACCENT)
    } else {
        (Color32::from_rgb(39, 45, 46), MUTED_TEXT)
    };

    egui::Frame::new()
        .fill(background)
        .corner_radius(4.0)
        .inner_margin(egui::Margin::symmetric(6, 3))
        .show(ui, |ui| {
            ui.label(RichText::new(name).small().strong().color(foreground));
        });
}
