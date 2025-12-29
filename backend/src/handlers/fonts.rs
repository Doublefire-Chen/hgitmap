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
/// These are cross-platform safe fonts that work on most systems
pub async fn get_available_fonts() -> impl Responder {
    let fonts = vec![
        // Sans-serif fonts (most common and safe)
        FontInfo {
            name: "Arial".to_string(),
            display_name: "Arial".to_string(),
            category: "sans-serif".to_string(),
        },
        FontInfo {
            name: "Helvetica".to_string(),
            display_name: "Helvetica".to_string(),
            category: "sans-serif".to_string(),
        },
        FontInfo {
            name: "Verdana".to_string(),
            display_name: "Verdana".to_string(),
            category: "sans-serif".to_string(),
        },
        FontInfo {
            name: "Tahoma".to_string(),
            display_name: "Tahoma".to_string(),
            category: "sans-serif".to_string(),
        },
        FontInfo {
            name: "Trebuchet MS".to_string(),
            display_name: "Trebuchet MS".to_string(),
            category: "sans-serif".to_string(),
        },
        // Serif fonts
        FontInfo {
            name: "Times New Roman".to_string(),
            display_name: "Times New Roman".to_string(),
            category: "serif".to_string(),
        },
        FontInfo {
            name: "Georgia".to_string(),
            display_name: "Georgia".to_string(),
            category: "serif".to_string(),
        },
        // Monospace fonts
        FontInfo {
            name: "Courier New".to_string(),
            display_name: "Courier New".to_string(),
            category: "monospace".to_string(),
        },
        FontInfo {
            name: "Courier".to_string(),
            display_name: "Courier".to_string(),
            category: "monospace".to_string(),
        },
        FontInfo {
            name: "monospace".to_string(),
            display_name: "System Monospace".to_string(),
            category: "monospace".to_string(),
        },
    ];

    HttpResponse::Ok().json(AvailableFontsResponse {
        fonts,
        default_font: "Arial".to_string(),
    })
}
