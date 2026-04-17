use eframe::egui::{
    self, Color32, CornerRadius, FontFamily, FontId, Frame, Margin, RichText, Stroke, Style,
    TextStyle, Vec2,
};

pub const BG: Color32 = Color32::from_rgb(241, 236, 228);
pub const SURFACE: Color32 = Color32::from_rgb(252, 249, 242);
pub const SURFACE_ELEVATED: Color32 = Color32::from_rgb(255, 252, 247);
pub const SURFACE_TINT: Color32 = Color32::from_rgb(234, 244, 241);
pub const ACCENT: Color32 = Color32::from_rgb(20, 122, 112);
pub const ACCENT_STRONG: Color32 = Color32::from_rgb(12, 98, 90);
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(201, 231, 224);
pub const TEXT: Color32 = Color32::from_rgb(31, 41, 55);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(104, 116, 136);
pub const BORDER: Color32 = Color32::from_rgb(221, 214, 203);
pub const WARNING: Color32 = Color32::from_rgb(193, 107, 27);
pub const ERROR: Color32 = Color32::from_rgb(179, 61, 58);
pub const SUCCESS: Color32 = Color32::from_rgb(38, 135, 92);

pub fn apply(ctx: &egui::Context) {
    let mut style: Style = (*ctx.global_style()).clone();
    style.spacing.item_spacing = Vec2::new(12.0, 12.0);
    style.spacing.button_padding = Vec2::new(16.0, 11.0);
    style.spacing.indent = 20.0;
    style.spacing.interact_size = Vec2::new(42.0, 36.0);
    style.visuals = visuals();
    style.text_styles = [
        (
            TextStyle::Heading,
            FontId::new(28.0, FontFamily::Proportional),
        ),
        (TextStyle::Body, FontId::new(16.0, FontFamily::Proportional)),
        (
            TextStyle::Button,
            FontId::new(15.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Monospace,
            FontId::new(14.0, FontFamily::Monospace),
        ),
        (
            TextStyle::Small,
            FontId::new(12.5, FontFamily::Proportional),
        ),
    ]
    .into();

    ctx.set_global_style(style);
}

fn visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::light();
    visuals.override_text_color = Some(TEXT);
    visuals.panel_fill = BG;
    visuals.window_fill = SURFACE_ELEVATED;
    visuals.faint_bg_color = Color32::from_rgb(236, 231, 223);
    visuals.extreme_bg_color = Color32::from_rgb(237, 234, 226);
    visuals.text_edit_bg_color = Some(Color32::from_rgb(248, 245, 239));
    visuals.code_bg_color = Color32::from_rgb(235, 242, 240);
    visuals.warn_fg_color = WARNING;
    visuals.error_fg_color = ERROR;
    visuals.hyperlink_color = ACCENT_STRONG;
    visuals.window_corner_radius = CornerRadius::same(22);
    visuals.menu_corner_radius = CornerRadius::same(18);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 10],
        blur: 28,
        spread: 0,
        color: Color32::from_black_alpha(22),
    };
    visuals.popup_shadow = egui::epaint::Shadow {
        offset: [0, 8],
        blur: 22,
        spread: 0,
        color: Color32::from_black_alpha(18),
    };
    visuals.window_stroke = Stroke::new(1.0, BORDER);
    visuals.selection.bg_fill = ACCENT;
    visuals.selection.stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.noninteractive = widget_visuals(SURFACE, BORDER, TEXT, 18);
    visuals.widgets.inactive = widget_visuals(SURFACE_ELEVATED, BORDER, TEXT, 16);
    visuals.widgets.hovered = widget_visuals(
        Color32::from_rgb(244, 250, 248),
        Color32::from_rgb(159, 196, 189),
        TEXT,
        16,
    );
    visuals.widgets.active = widget_visuals(ACCENT_SOFT, ACCENT, TEXT, 16);
    visuals.widgets.open = widget_visuals(SURFACE_TINT, ACCENT, TEXT, 16);
    visuals
}

fn widget_visuals(
    fill: Color32,
    stroke_color: Color32,
    text_color: Color32,
    radius: u8,
) -> egui::style::WidgetVisuals {
    egui::style::WidgetVisuals {
        bg_fill: fill,
        weak_bg_fill: fill,
        bg_stroke: Stroke::new(1.0, stroke_color),
        corner_radius: CornerRadius::same(radius),
        fg_stroke: Stroke::new(1.0, text_color),
        expansion: 0.0,
    }
}

pub fn app_canvas() -> Frame {
    Frame::new()
        .fill(BG)
        .inner_margin(Margin::same(28))
        .corner_radius(CornerRadius::same(28))
}

pub fn hero_card() -> Frame {
    Frame::new()
        .fill(SURFACE_TINT)
        .stroke(Stroke::new(1.0, Color32::from_rgb(187, 216, 209)))
        .inner_margin(Margin::same(22))
        .corner_radius(CornerRadius::same(24))
}

pub fn card() -> Frame {
    Frame::new()
        .fill(SURFACE)
        .stroke(Stroke::new(1.0, BORDER))
        .inner_margin(Margin::same(18))
        .corner_radius(CornerRadius::same(22))
}

pub fn card_emphasis() -> Frame {
    Frame::new()
        .fill(SURFACE_ELEVATED)
        .stroke(Stroke::new(1.0, Color32::from_rgb(206, 198, 185)))
        .inner_margin(Margin::same(22))
        .corner_radius(CornerRadius::same(24))
}

pub fn metric_card() -> Frame {
    Frame::new()
        .fill(Color32::from_rgb(247, 243, 235))
        .stroke(Stroke::new(1.0, BORDER))
        .inner_margin(Margin::same(16))
        .corner_radius(CornerRadius::same(20))
}

pub fn primary_button(text: impl Into<egui::WidgetText>) -> egui::Button<'static> {
    egui::Button::new(text)
        .fill(ACCENT)
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(16))
}

pub fn secondary_button(text: impl Into<egui::WidgetText>) -> egui::Button<'static> {
    egui::Button::new(text)
        .fill(SURFACE_ELEVATED)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(16))
}

pub fn ghost_button(text: impl Into<egui::WidgetText>) -> egui::Button<'static> {
    egui::Button::new(text)
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(16))
}

pub fn title(text: &str) -> RichText {
    RichText::new(text).size(30.0).strong().color(TEXT)
}

pub fn section_title(text: &str) -> RichText {
    RichText::new(text).size(21.0).strong().color(TEXT)
}

pub fn eyebrow(text: &str) -> RichText {
    RichText::new(text).size(12.0).strong().color(ACCENT_STRONG)
}

pub fn muted(text: &str) -> RichText {
    RichText::new(text).size(14.0).color(TEXT_MUTED)
}

pub fn field_label(text: &str) -> RichText {
    RichText::new(text).size(12.0).strong().color(TEXT_MUTED)
}

pub fn metric_value(text: &str) -> RichText {
    RichText::new(text).size(24.0).strong().color(TEXT)
}

pub fn status_fill(kind: crate::permission_diagnostics::PermissionStatus) -> Color32 {
    match kind {
        crate::permission_diagnostics::PermissionStatus::Ok => Color32::from_rgb(220, 239, 228),
        crate::permission_diagnostics::PermissionStatus::Info => Color32::from_rgb(222, 236, 245),
        crate::permission_diagnostics::PermissionStatus::Warning => {
            Color32::from_rgb(249, 232, 204)
        }
        crate::permission_diagnostics::PermissionStatus::Error => Color32::from_rgb(247, 219, 217),
    }
}

pub fn status_text(kind: crate::permission_diagnostics::PermissionStatus) -> Color32 {
    match kind {
        crate::permission_diagnostics::PermissionStatus::Ok => SUCCESS,
        crate::permission_diagnostics::PermissionStatus::Info => Color32::from_rgb(40, 90, 132),
        crate::permission_diagnostics::PermissionStatus::Warning => WARNING,
        crate::permission_diagnostics::PermissionStatus::Error => ERROR,
    }
}
