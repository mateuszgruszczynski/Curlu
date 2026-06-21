use eframe::egui;
use egui::text::LayoutJob;
use egui::{Color32, TextFormat};

use crate::theme::Palette;

pub fn json_colored(text: &str, font_id: egui::FontId, pal: &Palette) -> LayoutJob {
    json_impl(
        text, font_id,
        pal.syn_key, pal.syn_str, pal.syn_num,
        pal.syn_bool, pal.syn_null, pal.syn_punct,
    )
}

pub fn headers_colored(
    text: &str,
    font_id: egui::FontId,
    col_key: Color32,
    col_value: Color32,
    col_punct: Color32,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    let col_status = col_key;

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
            job.append(name, 0.0, fmt(col_key));
            job.append(":", 0.0, fmt(col_punct));
            job.append(value, 0.0, fmt(col_value));
        } else {
            job.append(line, 0.0, fmt(col_punct));
        }
    }
    job
}

fn json_impl(
    text: &str,
    font_id: egui::FontId,
    col_key: Color32,
    col_string: Color32,
    col_number: Color32,
    col_bool: Color32,
    col_null: Color32,
    col_punct: Color32,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;

    let fmt = |color: Color32| TextFormat {
        font_id: font_id.clone(),
        color,
        ..Default::default()
    };

    let mut context_stack: Vec<bool> = Vec::new(); // true = object, false = array
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
            ':' => { job.append(":", 0.0, fmt(col_punct)); i += 1; }
            ',' => { job.append(",", 0.0, fmt(col_punct)); i += 1; }
            '"' => {
                let start = i;
                i += 1;
                while i < len && chars[i] != '"' {
                    if chars[i] == '\\' { i += 1; }
                    i += 1;
                }
                if i < len { i += 1; }
                let s: String = chars[start..i].iter().collect();

                let is_key = in_object(&context_stack) && {
                    let mut j = i;
                    while j < len && chars[j].is_ascii_whitespace() { j += 1; }
                    j < len && chars[j] == ':'
                };
                job.append(&s, 0.0, fmt(if is_key { col_key } else { col_string }));
            }
            c if c.is_ascii_digit() || c == '-' => {
                let start = i;
                i += 1;
                while i < len && (chars[i].is_ascii_digit() || matches!(chars[i], '.' | 'e' | 'E' | '+' | '-')) {
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
                while i < len && !matches!(chars[i], '{' | '}' | '[' | ']' | '"' | ':' | ',' | 't' | 'f' | 'n' | '-' | '0'..='9') {
                    i += 1;
                }
                if i == start { i += 1; }
                let s: String = chars[start..i].iter().collect();
                job.append(&s, 0.0, fmt(col_punct));
            }
        }
    }
    job
}
