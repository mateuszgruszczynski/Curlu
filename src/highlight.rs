use eframe::egui;
use egui::text::LayoutJob;
use egui::{Color32, TextFormat};

use crate::theme;

pub fn json(text: &str, font_id: egui::FontId) -> LayoutJob {
    let mut job = LayoutJob::default();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;

    let col_key = theme::COLOR_KEY;
    let col_string = theme::COLOR_STRING;
    let col_number = theme::COLOR_NUMBER;
    let col_bool = theme::COLOR_KEYWORD;
    let col_null = theme::COLOR_KEYWORD;
    let col_punct = theme::COLOR_PUNCTUATION;

    let fmt = |color: Color32| TextFormat {
        font_id: font_id.clone(),
        color,
        ..Default::default()
    };

    // Track whether we're inside an object (expect keys) or array (expect values)
    // true = object context, false = array context
    let mut context_stack: Vec<bool> = Vec::new();

    let in_object = |stack: &[bool]| stack.last().copied().unwrap_or(false);

    while i < len {
        let c = chars[i];
        match c {
            '{' => {
                job.append(&String::from(c), 0.0, fmt(col_punct));
                context_stack.push(true);
                i += 1;
            }
            '[' => {
                job.append(&String::from(c), 0.0, fmt(col_punct));
                context_stack.push(false);
                i += 1;
            }
            '}' | ']' => {
                job.append(&String::from(c), 0.0, fmt(col_punct));
                context_stack.pop();
                i += 1;
            }
            ':' => {
                job.append(":", 0.0, fmt(col_punct));
                i += 1;
            }
            ',' => {
                job.append(",", 0.0, fmt(col_punct));
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

                // In object context, a string followed by ':' is a key
                let is_key = in_object(&context_stack) && {
                    let mut j = i;
                    while j < len && chars[j].is_ascii_whitespace() {
                        j += 1;
                    }
                    j < len && chars[j] == ':'
                };

                let color = if is_key { col_key } else { col_string };
                job.append(&s, 0.0, fmt(color));
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

    let col_name = theme::COLOR_KEY;
    let col_value = theme::COLOR_STRING;
    let col_status = theme::COLOR_KEYWORD;
    let col_punct = theme::COLOR_PUNCTUATION;

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
