//! High performance 2D renderer for the Reflex HUD pill matching Hex's design system.

use crate::icons::LucideIcon;
use fontdue::{Font, FontSettings};
use reflex_core::hud::{HudEvent, HudIcon, HudTone};
use std::fs;
use std::path::{Path, PathBuf};
use tiny_skia::{Color, Paint, PathBuilder, Pixmap, PixmapMut, Stroke, Transform};

pub struct FontContext {
    pub bold: Option<Font>,
    pub regular: Option<Font>,
}

impl FontContext {
    pub fn load_system_fonts() -> Self {
        let candidates_bold = [
            "/usr/share/fonts/noto/NotoSans-Bold.ttf",
            "/usr/share/fonts/TTF/NotoSans-Bold.ttf",
            "/usr/share/fonts/TTF/JetBrainsMonoNLNerdFont-SemiBold.ttf",
            "/usr/share/fonts/truetype/noto/NotoSans-Bold.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
            "/usr/share/fonts/dejavu/DejaVuSans-Bold.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
            "/run/current-system/sw/share/fonts/noto/NotoSans-Bold.ttf",
            "/run/current-system/sw/share/fonts/truetype/noto/NotoSans-Bold.ttf",
        ];

        let candidates_regular = [
            "/usr/share/fonts/noto/NotoSans-Regular.ttf",
            "/usr/share/fonts/TTF/NotoSans-Regular.ttf",
            "/usr/share/fonts/TTF/JetBrainsMonoNLNerdFontMono-Regular.ttf",
            "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            "/run/current-system/sw/share/fonts/noto/NotoSans-Regular.ttf",
            "/run/current-system/sw/share/fonts/truetype/noto/NotoSans-Regular.ttf",
        ];

        let mut bold = candidates_bold
            .iter()
            .find_map(|path| fs::read(Path::new(path)).ok())
            .and_then(|bytes| Font::from_bytes(bytes, FontSettings::default()).ok());

        let mut regular = candidates_regular
            .iter()
            .find_map(|path| fs::read(Path::new(path)).ok())
            .and_then(|bytes| Font::from_bytes(bytes, FontSettings::default()).ok());

        // Fallback: discover any valid TrueType/OpenType font in /usr/share/fonts if hardcoded paths missed
        if bold.is_none() || regular.is_none() {
            if let Some(fallback_bytes) = find_any_system_font() {
                if let Ok(font) = Font::from_bytes(fallback_bytes, FontSettings::default()) {
                    if bold.is_none() {
                        bold = Some(font.clone());
                    }
                    if regular.is_none() {
                        regular = Some(font);
                    }
                }
            }
        }

        Self { bold, regular }
    }
}

fn find_any_system_font() -> Option<Vec<u8>> {
    let font_dirs = [
        "/usr/share/fonts",
        "/run/current-system/sw/share/fonts",
        "/usr/local/share/fonts",
    ];

    for dir in &font_dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            let mut stack: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
            while let Some(path) = stack.pop() {
                if path.is_dir() {
                    if let Ok(sub) = fs::read_dir(&path) {
                        for sub_entry in sub.flatten() {
                            stack.push(sub_entry.path());
                        }
                    }
                } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if (ext.eq_ignore_ascii_case("ttf") || ext.eq_ignore_ascii_case("otf"))
                        && !path.to_string_lossy().contains("Emoji")
                    {
                        if let Ok(bytes) = fs::read(&path) {
                            return Some(bytes);
                        }
                    }
                }
            }
        }
    }
    None
}

#[derive(Debug, Clone, Copy)]
pub enum PillTone {
    Emerald, // Success / Deterministic action
    Cyan,    // Navigation / Workspace
    Amber,   // Warning / Guardrail limit
    Coral,   // Confirmation / Protected target
    Purple,  // AI Fallback
}

impl PillTone {
    pub fn hex(&self) -> &'static str {
        match self {
            PillTone::Emerald => "#10B981",
            PillTone::Cyan => "#38BDF8",
            PillTone::Amber => "#F59E0B",
            PillTone::Coral => "#FB7185",
            PillTone::Purple => "#C084FC",
        }
    }

    pub fn rgb(&self) -> (u8, u8, u8) {
        match self {
            PillTone::Emerald => (16, 185, 129),
            PillTone::Cyan => (56, 189, 248),
            PillTone::Amber => (245, 158, 11),
            PillTone::Coral => (251, 113, 133),
            PillTone::Purple => (192, 132, 252),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PillItem {
    pub icon: LucideIcon,
    pub title: String,
    pub detail: Option<String>,
    pub tone: PillTone,
}

impl From<HudTone> for PillTone {
    fn from(tone: HudTone) -> Self {
        match tone {
            HudTone::Emerald => PillTone::Emerald,
            HudTone::Cyan => PillTone::Cyan,
            HudTone::Amber => PillTone::Amber,
            HudTone::Coral => PillTone::Coral,
            HudTone::Purple => PillTone::Purple,
        }
    }
}

impl From<HudIcon> for LucideIcon {
    fn from(icon: HudIcon) -> Self {
        match icon {
            HudIcon::Volume2 => LucideIcon::Volume2,
            HudIcon::VolumeX => LucideIcon::VolumeX,
            HudIcon::Volume1 => LucideIcon::Volume1,
            HudIcon::Layers => LucideIcon::Layers,
            HudIcon::Move => LucideIcon::Move,
            HudIcon::Pin => LucideIcon::Pin,
            HudIcon::Maximize2 => LucideIcon::Maximize2,
            HudIcon::ShieldAlert => LucideIcon::ShieldAlert,
            HudIcon::Lock => LucideIcon::Lock,
            HudIcon::Clipboard => LucideIcon::Clipboard,
            HudIcon::Camera => LucideIcon::Camera,
            HudIcon::Headphones => LucideIcon::Headphones,
            HudIcon::Speaker => LucideIcon::Speaker,
            HudIcon::Zap => LucideIcon::Zap,
            HudIcon::Check => LucideIcon::Check,
            HudIcon::AlertTriangle => LucideIcon::AlertTriangle,
            HudIcon::Sparkles => LucideIcon::Sparkles,
        }
    }
}

impl From<HudEvent> for PillItem {
    fn from(event: HudEvent) -> Self {
        Self {
            icon: event.icon.into(),
            title: event.title,
            detail: event.detail,
            tone: event.tone.into(),
        }
    }
}

pub fn render_pill(
    pixmap: &mut PixmapMut,
    fonts: &FontContext,
    item: &PillItem,
    spring_scale: f32,
    alpha: f32,
) {
    pixmap.fill(Color::TRANSPARENT);

    if spring_scale <= 0.05 || alpha <= 0.01 {
        return;
    }

    let canvas_w = pixmap.width() as f32;
    let canvas_h = pixmap.height() as f32;

    // 1. Measure natural content width for auto-fitting pill
    let icon_size = 16.0;
    let pad_left = 14.0;
    let gap_icon_text = 10.0;
    let font_size_title = 13.0;
    let font_size_detail = 12.0;
    let pad_right = 14.0;
    let max_w = (canvas_w - 28.0).max(180.0);

    let font_bold = fonts.bold.as_ref().or(fonts.regular.as_ref());
    let font_regular = fonts.regular.as_ref().or(fonts.bold.as_ref());

    let measure_str = |text: &str, font: Option<&Font>, size: f32| -> f32 {
        if let Some(f) = font {
            text.chars().map(|c| f.rasterize(c, size).0.advance_width).sum()
        } else {
            text.len() as f32 * 7.0
        }
    };

    let title_w = measure_str(&item.title, font_bold, font_size_title);

    // Dynamic truncation if content exceeds max_w
    let mut display_detail = item.detail.clone();
    let mut gap_title_detail = if display_detail.is_some() { 8.0 } else { 0.0 };
    let mut detail_w = display_detail.as_ref().map_or(0.0, |d| measure_str(d, font_regular, font_size_detail));

    let mut content_w = pad_left + icon_size + gap_icon_text + title_w + gap_title_detail + detail_w + pad_right;

    // If overflowing, truncate detail text with ellipsis
    if content_w > max_w && display_detail.is_some() {
        if let Some(ref mut d) = display_detail {
            while !d.is_empty() && (pad_left + icon_size + gap_icon_text + title_w + gap_title_detail + detail_w + pad_right) > max_w {
                d.pop();
                let candidate = format!("{}…", d);
                detail_w = measure_str(&candidate, font_regular, font_size_detail);
            }
            if !d.is_empty() {
                *d = format!("{}…", d);
            } else {
                display_detail = None;
                gap_title_detail = 0.0;
                detail_w = 0.0;
            }
            content_w = pad_left + icon_size + gap_icon_text + title_w + gap_title_detail + detail_w + pad_right;
        }
    }

    let natural_w = content_w.clamp(140.0, max_w);
    let natural_h = 36.0; // Hex signature compact plate height

    // Apply exact spring scale transform
    let pill_w = natural_w * spring_scale;
    let pill_h = natural_h * spring_scale;
    let pill_x = (canvas_w - pill_w) / 2.0;

    // Intentional soft vertical retraction: float slightly upward into the top bezel during exit
    let exit_offset = (1.0 - alpha) * -4.0;
    let pill_y = (canvas_h - pill_h) / 2.0 + exit_offset;
    let radius = pill_h / 2.0;

    // 2. Analytic Soft Drop Shadow (Hex formula: rgba(0, 0, 0, 0.50))
    if let Some(shadow_path) = rounded_rect_path(pill_x, pill_y + 3.0, pill_w, pill_h, radius) {
        let mut shadow_paint = Paint::default();
        shadow_paint.set_color_rgba8(0, 0, 0, (110.0 * alpha) as u8);
        shadow_paint.anti_alias = true;
        pixmap.fill_path(
            &shadow_path,
            &shadow_paint,
            tiny_skia::FillRule::Winding,
            Transform::identity(),
            None,
        );
    }

    // 3. Hex Dark Obsidian Plate: rgba(17, 17, 20, 0.94)
    if let Some(body_path) = rounded_rect_path(pill_x, pill_y, pill_w, pill_h, radius) {
        let mut body_paint = Paint::default();
        body_paint.set_color_rgba8(17, 17, 20, (240.0 * alpha) as u8);
        body_paint.anti_alias = true;
        pixmap.fill_path(
            &body_path,
            &body_paint,
            tiny_skia::FillRule::Winding,
            Transform::identity(),
            None,
        );

        // 4. Hex Micro-Rim Highlight: rgba(255, 255, 255, 0.12) + faint tone accent
        let (tr, tg, tb) = item.tone.rgb();
        let mut rim_paint = Paint::default();
        rim_paint.set_color_rgba8(255, 255, 255, (32.0 * alpha) as u8);
        rim_paint.anti_alias = true;

        let stroke = Stroke {
            width: 1.0,
            ..Default::default()
        };
        pixmap.stroke_path(
            &body_path,
            &rim_paint,
            &stroke,
            Transform::identity(),
            None,
        );

        // Subtle accent tone highlight along the border
        let mut glow_paint = Paint::default();
        glow_paint.set_color_rgba8(tr, tg, tb, (45.0 * alpha) as u8);
        glow_paint.anti_alias = true;
        pixmap.stroke_path(
            &body_path,
            &glow_paint,
            &stroke,
            Transform::identity(),
            None,
        );
    }

    // Hide inner contents while compressed below 0.65 to ensure pristine presentation
    if spring_scale < 0.65 {
        return;
    }

    let center_y = pill_y + pill_h / 2.0;
    let mut cursor_x = pill_x + pad_left;

    // 5. Lucide React Vector Icon (via resvg)
    let icon_px = icon_size as u32;
    let icon_svg = item.icon.to_svg_string(item.tone.hex(), icon_px);
    let opt = resvg::usvg::Options::default();
    if let Ok(tree) = resvg::usvg::Tree::from_str(&icon_svg, &opt) {
        let icon_y = center_y - icon_size / 2.0;
        if let Some(mut icon_pixmap) = Pixmap::new(icon_px, icon_px) {
            icon_pixmap.fill(Color::TRANSPARENT);
            resvg::render(&tree, Transform::identity(), &mut icon_pixmap.as_mut());

            let paint = tiny_skia::PixmapPaint {
                opacity: alpha,
                ..Default::default()
            };
            pixmap.draw_pixmap(
                cursor_x as i32,
                icon_y as i32,
                icon_pixmap.as_ref(),
                &paint,
                Transform::identity(),
                None,
            );
        }
    }
    cursor_x += icon_size + gap_icon_text;

    // 6. Title Text: Pure White (#F4F4F5), Optical Centered
    if let Some(font) = font_bold {
        let baseline = center_y + 4.2;
        let text_color = (244, 244, 245, (255.0 * alpha) as u8);

        for ch in item.title.chars() {
            let (metrics, bitmap) = font.rasterize(ch, font_size_title);
            let gx = cursor_x + metrics.xmin as f32;
            let gy = baseline - metrics.ymin as f32 - metrics.height as f32;

            draw_glyph_bitmap(pixmap, &bitmap, gx as i32, gy as i32, metrics.width, metrics.height, text_color);
            cursor_x += metrics.advance_width;
        }
    }

    // 7. Detail Text (if present): Subtle Zinc (#A1A1AA)
    if let Some(ref detail) = display_detail {
        if let Some(font) = font_regular {
            cursor_x += gap_title_detail;
            let baseline = center_y + 3.8;
            let detail_color = (161, 161, 170, (190.0 * alpha) as u8);

            for ch in detail.chars() {
                let (metrics, bitmap) = font.rasterize(ch, font_size_detail);
                let gx = cursor_x + metrics.xmin as f32;
                let gy = baseline - metrics.ymin as f32 - metrics.height as f32;

                draw_glyph_bitmap(pixmap, &bitmap, gx as i32, gy as i32, metrics.width, metrics.height, detail_color);
                cursor_x += metrics.advance_width;
            }
        }
    }
}

#[allow(clippy::manual_checked_ops)]
fn draw_glyph_bitmap(
    pixmap: &mut PixmapMut,
    bitmap: &[u8],
    gx: i32,
    gy: i32,
    width: usize,
    height: usize,
    color: (u8, u8, u8, u8),
) {
    let pw = pixmap.width() as i32;
    let ph = pixmap.height() as i32;
    let (r, g, b, base_alpha) = color;

    for row in 0..height {
        let dest_y = gy + row as i32;
        if dest_y < 0 || dest_y >= ph {
            continue;
        }

        for col in 0..width {
            let dest_x = gx + col as i32;
            if dest_x < 0 || dest_x >= pw {
                continue;
            }

            let coverage = bitmap[row * width + col];
            if coverage == 0 {
                continue;
            }

            let a = ((coverage as u32 * base_alpha as u32) / 255) as u8;
            if a == 0 {
                continue;
            }

            let pixel_slice = pixmap.pixels_mut();
            let idx = (dest_y as usize * pw as usize) + dest_x as usize;
            let existing = pixel_slice[idx];

            let ea = existing.alpha();
            let er = existing.red();
            let eg = existing.green();
            let eb = existing.blue();

            let out_a = a as u32 + (ea as u32 * (255 - a as u32)) / 255;
            if out_a > 0 {
                let out_r = ((r as u32 * a as u32) + (er as u32 * ea as u32 * (255 - a as u32) / 255)) / out_a;
                let out_g = ((g as u32 * a as u32) + (eg as u32 * ea as u32 * (255 - a as u32) / 255)) / out_a;
                let out_b = ((b as u32 * a as u32) + (eb as u32 * ea as u32 * (255 - a as u32) / 255)) / out_a;

                if let Some(color) = tiny_skia::PremultipliedColorU8::from_rgba(
                    out_r as u8,
                    out_g as u8,
                    out_b as u8,
                    out_a as u8,
                ) {
                    pixel_slice[idx] = color;
                }
            }
        }
    }
}

pub fn rounded_rect_path(x: f32, y: f32, w: f32, h: f32, r: f32) -> Option<tiny_skia::Path> {
    let mut pb = PathBuilder::new();
    let r = r.min(w / 2.0).min(h / 2.0);

    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.quad_to(x + w, y, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.quad_to(x + w, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.quad_to(x, y + h, x, y + h - r);
    pb.line_to(x, y + r);
    pb.quad_to(x, y, x + r, y);
    pb.close();
    pb.finish()
}
