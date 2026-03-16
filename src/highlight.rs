use eframe::egui;
use egui::text::LayoutJob;
use egui::{Color32, TextFormat};

pub fn json(text: &str, font_id: egui::FontId) -> LayoutJob {
    let mut job = LayoutJob::default();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;

    let col_key = Color32::from_rgb(156, 220, 254);
    let col_string = Color32::from_rgb(206, 145, 120);
    let col_number = Color32::from_rgb(181, 206, 168);
    let col_bool = Color32::from_rgb(86, 156, 214);
    let col_null = Color32::from_rgb(86, 156, 214);
    let col_punct = Color32::from_rgb(212, 212, 212);

    let fmt = |color: Color32| TextFormat {
        font_id: font_id.clone(),
        color,
        ..Default::default()
    };

    let mut expect_key = false;

    while i < len {
        let c = chars[i];
        match c {
            '{' | '[' => {
                job.append(&String::from(c), 0.0, fmt(col_punct));
                expect_key = c == '{';
                i += 1;
            }
            '}' | ']' => {
                job.append(&String::from(c), 0.0, fmt(col_punct));
                expect_key = false;
                i += 1;
            }
            ':' => {
                job.append(":", 0.0, fmt(col_punct));
                expect_key = false;
                i += 1;
            }
            ',' => {
                job.append(",", 0.0, fmt(col_punct));
                expect_key = true;
                i += 1;
            }
            '"' => {
                let start = i;
                i += 1;
                while i < len && chars[i] != '"' {
                    if chars[i] == '\\' {
                        i += 1;
                    }
                    i += 1;
                }
                if i < len {
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                let color = if expect_key { col_key } else { col_string };
                job.append(&s, 0.0, fmt(color));
                if expect_key {
                    expect_key = false;
                }
            }
            c if c.is_ascii_digit() || c == '-' => {
                let start = i;
                i += 1;
                while i < len
                    && (chars[i].is_ascii_digit()
                        || chars[i] == '.'
                        || chars[i] == 'e'
                        || chars[i] == 'E'
                        || chars[i] == '+'
                        || chars[i] == '-')
                {
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                job.append(&s, 0.0, fmt(col_number));
            }
            't' | 'f' if text[i..].starts_with("true") || text[i..].starts_with("false") => {
                let word = if chars[i] == 't' { "true" } else { "false" };
                job.append(word, 0.0, fmt(col_bool));
                i += word.len();
            }
            'n' if text[i..].starts_with("null") => {
                job.append("null", 0.0, fmt(col_null));
                i += 4;
            }
            _ => {
                let start = i;
                while i < len
                    && !matches!(
                        chars[i],
                        '{' | '}' | '[' | ']' | '"' | ':' | ',' | 't' | 'f' | 'n' | '-' | '0'..='9'
                    )
                {
                    i += 1;
                }
                if i == start {
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                job.append(&s, 0.0, fmt(col_punct));
            }
        }
    }

    job
}

pub fn headers(text: &str, font_id: egui::FontId) -> LayoutJob {
    let mut job = LayoutJob::default();

    let col_name = Color32::from_rgb(156, 220, 254);
    let col_value = Color32::from_rgb(206, 145, 120);
    let col_status = Color32::from_rgb(86, 156, 214);
    let col_punct = Color32::from_rgb(212, 212, 212);

    let fmt = |color: Color32| TextFormat {
        font_id: font_id.clone(),
        color,
        ..Default::default()
    };

    for (line_idx, line) in text.split('\n').enumerate() {
        if line_idx > 0 {
            job.append("\n", 0.0, fmt(col_punct));
        }
        if line.starts_with("HTTP") {
            job.append(line, 0.0, fmt(col_status));
        } else if let Some((name, value)) = line.split_once(':') {
            job.append(name, 0.0, fmt(col_name));
            job.append(":", 0.0, fmt(col_punct));
            job.append(value, 0.0, fmt(col_value));
        } else {
            job.append(line, 0.0, fmt(col_punct));
        }
    }

    job
}
