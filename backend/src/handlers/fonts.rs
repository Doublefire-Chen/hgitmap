use actix_web::{HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FontInfo {
    pub name: String,
    pub display_name: String,
    pub category: String, // "sans-serif", "serif", "monospace"
}

#[derive(Debug, Serialize)]
pub struct AvailableFontsResponse {
    pub fonts: Vec<FontInfo>,
    pub default_font: String,
}

/// Get list of available fonts for heatmap generation
/// These fonts are commonly available on Linux systems (Debian/Ubuntu)
pub async fn get_available_fonts() -> impl Responder {
    let fonts = vec![
        // Sans-serif fonts (Linux standard)
        FontInfo {
            name: "DejaVu Sans".to_string(),
            display_name: "DejaVu Sans".to_string(),
            category: "sans-serif".to_string(),
        },
        FontInfo {
            name: "Nimbus Sans".to_string(),
            display_name: "Nimbus Sans".to_string(),
            category: "sans-serif".to_string(),
        },
        FontInfo {
            name: "Nimbus Sans Narrow".to_string(),
            display_name: "Nimbus Sans Narrow".to_string(),
            category: "sans-serif".to_string(),
        },
        FontInfo {
            name: "URW Gothic".to_string(),
            display_name: "URW Gothic".to_string(),
            category: "sans-serif".to_string(),
        },
        FontInfo {
            name: "Droid Sans Fallback".to_string(),
            display_name: "Droid Sans".to_string(),
            category: "sans-serif".to_string(),
        },
        // Serif fonts
        FontInfo {
            name: "DejaVu Serif".to_string(),
            display_name: "DejaVu Serif".to_string(),
            category: "serif".to_string(),
        },
        FontInfo {
            name: "Nimbus Roman".to_string(),
            display_name: "Nimbus Roman".to_string(),
            category: "serif".to_string(),
        },
        FontInfo {
            name: "C059".to_string(),
            display_name: "C059 (Century Schoolbook)".to_string(),
            category: "serif".to_string(),
        },
        FontInfo {
            name: "P052".to_string(),
            display_name: "P052 (Palatino)".to_string(),
            category: "serif".to_string(),
        },
        FontInfo {
            name: "URW Bookman".to_string(),
            display_name: "URW Bookman".to_string(),
            category: "serif".to_string(),
        },
        // Monospace fonts
        FontInfo {
            name: "DejaVu Sans Mono".to_string(),
            display_name: "DejaVu Sans Mono".to_string(),
            category: "monospace".to_string(),
        },
        FontInfo {
            name: "Nimbus Mono PS".to_string(),
            display_name: "Nimbus Mono PS".to_string(),
            category: "monospace".to_string(),
        },
        FontInfo {
            name: "Noto Sans Mono".to_string(),
            display_name: "Noto Sans Mono".to_string(),
            category: "monospace".to_string(),
        },
        FontInfo {
            name: "Noto Mono".to_string(),
            display_name: "Noto Mono".to_string(),
            category: "monospace".to_string(),
        },
    ];

    HttpResponse::Ok().json(AvailableFontsResponse {
        fonts,
        default_font: "DejaVu Sans".to_string(),
    })
}
