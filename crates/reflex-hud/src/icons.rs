//! Lucide React vector icons rendered via resvg / tiny-skia.

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LucideIcon {
    Volume2,
    VolumeX,
    Volume1,
    Layers,
    Move,
    Pin,
    Maximize2,
    ShieldAlert,
    Lock,
    Clipboard,
    Camera,
    Headphones,
    Speaker,
    Zap,
    Check,
    AlertTriangle,
    Sparkles,
}

impl LucideIcon {
    pub fn inner_svg_elements(&self) -> &'static str {
        match self {
            LucideIcon::Volume2 => r#"<polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07"/><path d="M19.07 4.93a10 10 0 0 1 0 14.14"/>"#,
            LucideIcon::VolumeX => r#"<polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><line x1="22" x2="16" y1="9" y2="15"/><line x1="16" x2="22" y1="9" y2="15"/>"#,
            LucideIcon::Volume1 => r#"<polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07"/>"#,
            LucideIcon::Layers => r#"<polygon points="12 2 2 7 12 12 22 7 12 2"/><polyline points="2 17 12 22 22 17"/><polyline points="2 12 12 17 22 12"/>"#,
            LucideIcon::Move => r#"<polyline points="5 9 2 12 5 15"/><polyline points="9 5 12 2 15 5"/><polyline points="15 19 12 22 9 19"/><polyline points="19 9 22 12 19 15"/><line x1="2" x2="22" y1="12" y2="12"/><line x1="12" x2="12" y1="2" y2="22"/>"#,
            LucideIcon::Pin => r#"<line x1="12" x2="12" y1="17" y2="22"/><path d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V6h1a2 2 0 0 0 0-4H8a2 2 0 0 0 0 4h1v4.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24Z"/>"#,
            LucideIcon::Maximize2 => r#"<polyline points="15 3 21 3 21 9"/><polyline points="9 21 3 21 3 15"/><line x1="21" x2="14" y1="3" y2="10"/><line x1="3" x2="10" y1="21" y2="14"/>"#,
            LucideIcon::ShieldAlert => r#"<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10"/><line x1="12" x2="12" y1="8" y2="12"/><line x1="12" x2="12.01" y1="16" y2="16"/>"#,
            LucideIcon::Lock => r#"<rect width="18" height="11" x="3" y="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/>"#,
            LucideIcon::Clipboard => r#"<rect width="8" height="4" x="8" y="2" rx="1" ry="1"/><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/>"#,
            LucideIcon::Camera => r#"<path d="M14.5 4h-5L7 7H4a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-3l-2.5-3z"/><circle cx="12" cy="13" r="3"/>"#,
            LucideIcon::Headphones => r#"<path d="M3 14h3a2 2 0 0 1 2 2v3a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-7a9 9 0 0 1 18 0v7a2 2 0 0 1-2 2h-1a2 2 0 0 1-2-2v-3a2 2 0 0 1 2-2h3"/>"#,
            LucideIcon::Speaker => r#"<rect width="16" height="20" x="4" y="2" rx="2"/><circle cx="12" cy="14" r="4"/><line x1="12" x2="12.01" y1="6" y2="6"/>"#,
            LucideIcon::Zap => r#"<polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/>"#,
            LucideIcon::Check => r#"<polyline points="20 6 9 17 4 12"/>"#,
            LucideIcon::AlertTriangle => r#"<path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"/><line x1="12" x2="12" y1="9" y2="13"/><line x1="12" x2="12.01" y1="17" y2="17"/>"#,
            LucideIcon::Sparkles => r#"<path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z"/>"#,
        }
    }

    /// Build a complete standalone SVG XML string with the specified stroke color.
    pub fn to_svg_string(self, stroke_hex: &str, size: u32) -> String {
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 24 24" fill="none" stroke="{}" stroke-width="2.3" stroke-linecap="round" stroke-linejoin="round">{}</svg>"#,
            size,
            size,
            stroke_hex,
            self.inner_svg_elements()
        )
    }
}
