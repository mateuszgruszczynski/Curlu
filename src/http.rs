use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

impl Method {
    pub const ALL: &[Method] = &[
        Method::Get,
        Method::Post,
        Method::Put,
        Method::Patch,
        Method::Delete,
        Method::Head,
        Method::Options,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Patch => "PATCH",
            Method::Delete => "DELETE",
            Method::Head => "HEAD",
            Method::Options => "OPTIONS",
        }
    }
}

impl FromStr for Method {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Method::Get),
            "POST" => Ok(Method::Post),
            "PUT" => Ok(Method::Put),
            "PATCH" => Ok(Method::Patch),
            "DELETE" => Ok(Method::Delete),
            "HEAD" => Ok(Method::Head),
            "OPTIONS" => Ok(Method::Options),
            _ => Err(()),
        }
    }
}

pub struct SavedRequest {
    pub method: Method,
    pub url: String,
    pub headers: String,
    pub body: String,
}

impl SavedRequest {
    pub fn to_curl(&self) -> String {
        let mut parts = vec![format!("curl -X {}", self.method.as_str())];

        for line in self.headers.lines() {
            let line = line.trim();
            if !line.is_empty() {
                parts.push(format!("  -H '{}'", line.replace('\'', "'\\''")));
            }
        }

        if !self.body.is_empty() {
            parts.push(format!("  -d '{}'", self.body.replace('\'', "'\\''")));
        }

        parts.push(format!("  '{}'", self.url.replace('\'', "'\\''")));
        parts.join(" \\\n")
    }

    pub fn from_curl(text: &str) -> Option<SavedRequest> {
        let joined = text
            .lines()
            .map(|l| l.trim())
            .collect::<Vec<_>>()
            .join(" ");

        let joined = joined.replace(" \\ ", " ");

        let tokens = shell_tokenize(&joined);
        if tokens.first().map(|s| s.as_str()) != Some("curl") {
            return None;
        }

        let mut method = Method::Get;
        let mut url = String::new();
        let mut headers = Vec::new();
        let mut body = String::new();

        let mut i = 1;
        while i < tokens.len() {
            match tokens[i].as_str() {
                "-X" | "--request" => {
                    i += 1;
                    if i < tokens.len() {
                        method = tokens[i].parse().unwrap_or(Method::Get);
                    }
                }
                "-H" | "--header" => {
                    i += 1;
                    if i < tokens.len() {
                        headers.push(tokens[i].clone());
                    }
                }
                "-d" | "--data" | "--data-raw" => {
                    i += 1;
                    if i < tokens.len() {
                        body = tokens[i].clone();
                    }
                }
                other if !other.starts_with('-') => {
                    url = other.to_string();
                }
                _ => {}
            }
            i += 1;
        }

        Some(SavedRequest {
            method,
            url,
            headers: headers.join("\n"),
            body,
        })
    }
}

fn shell_tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
                chars.next();
            }
            '\'' => {
                chars.next();
                while let Some(&c) = chars.peek() {
                    if c == '\'' {
                        chars.next();
                        break;
                    }
                    current.push(c);
                    chars.next();
                }
            }
            '"' => {
                chars.next();
                while let Some(&c) = chars.peek() {
                    if c == '"' {
                        chars.next();
                        break;
                    }
                    if c == '\\' {
                        chars.next();
                        if let Some(&next) = chars.peek() {
                            current.push(next);
                            chars.next();
                        }
                    } else {
                        current.push(c);
                        chars.next();
                    }
                }
            }
            '\\' => {
                chars.next();
                if let Some(&next) = chars.peek() {
                    current.push(next);
                    chars.next();
                }
            }
            _ => {
                current.push(c);
                chars.next();
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

pub fn pretty_print_json(text: &str) -> Option<String> {
    let val: serde_json::Value = serde_json::from_str(text.trim()).ok()?;
    serde_json::to_string_pretty(&val).ok()
}

/// Apply a dot-notation JSON path to a JSON string, preserving the full key
/// hierarchy around the matched value(s). Supports `[*]` wildcard to map over
/// all array elements. Returns `None` if the body is not valid JSON or the
/// path does not match anything.
///
/// Examples: `data.user.id`, `data[*].id`, `results[0].name`, `$.items[*].tags[0]`
pub fn apply_json_filter(json_str: &str, path: &str) -> Option<String> {
    let root: serde_json::Value = serde_json::from_str(json_str.trim()).ok()?;
    let segments = parse_path_segments(path);
    let result = if segments.is_empty() {
        root
    } else {
        filter_recursive(&root, &segments)?
    };
    serde_json::to_string_pretty(&result).ok()
}

fn filter_recursive(value: &serde_json::Value, segments: &[PathSegment]) -> Option<serde_json::Value> {
    if segments.is_empty() {
        return Some(value.clone());
    }
    match &segments[0] {
        PathSegment::Key(k) => {
            let child = value.get(k.as_str())?;
            let filtered = filter_recursive(child, &segments[1..])?;
            let mut map = serde_json::Map::new();
            map.insert(k.clone(), filtered);
            Some(serde_json::Value::Object(map))
        }
        PathSegment::Index(i) => {
            let child = value.get(i)?;
            let filtered = filter_recursive(child, &segments[1..])?;
            Some(serde_json::Value::Array(vec![filtered]))
        }
        PathSegment::Wildcard => {
            let arr = value.as_array()?;
            let results: Vec<serde_json::Value> = arr
                .iter()
                .filter_map(|elem| filter_recursive(elem, &segments[1..]))
                .collect();
            if results.is_empty() { None } else { Some(serde_json::Value::Array(results)) }
        }
    }
}

enum PathSegment {
    Key(String),
    Index(usize),
    Wildcard,
}

fn parse_path_segments(path: &str) -> Vec<PathSegment> {
    let path = path.trim();
    let path = path.strip_prefix('$').unwrap_or(path);
    let path = path.trim_start_matches('.');
    if path.is_empty() {
        return Vec::new();
    }

    let mut segments = Vec::new();
    let mut segment = String::new();
    let mut chars = path.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '.' => {
                if !segment.is_empty() {
                    segments.push(PathSegment::Key(std::mem::take(&mut segment)));
                }
            }
            '[' => {
                if !segment.is_empty() {
                    segments.push(PathSegment::Key(std::mem::take(&mut segment)));
                }
                let mut idx = String::new();
                for inner in chars.by_ref() {
                    if inner == ']' { break; }
                    idx.push(inner);
                }
                if idx == "*" {
                    segments.push(PathSegment::Wildcard);
                } else if let Ok(i) = idx.parse::<usize>() {
                    segments.push(PathSegment::Index(i));
                }
                if chars.peek() == Some(&'.') {
                    chars.next();
                }
            }
            other => segment.push(other),
        }
    }
    if !segment.is_empty() {
        segments.push(PathSegment::Key(segment));
    }
    segments
}

pub struct Response {
    pub headers: String,
    pub body: String,
    pub status: u16,
    pub elapsed_ms: f64,
    pub body_bytes: usize,
}

pub fn send_request(method: Method, url: &str, raw_headers: &str, body: &str) -> Response {
    let start = std::time::Instant::now();
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap_or_else(|_| Client::new());

    let mut headers = HeaderMap::new();
    for line in raw_headers.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            if let (Ok(name), Ok(val)) = (HeaderName::from_str(key), HeaderValue::from_str(value))
            {
                headers.insert(name, val);
            }
        }
    }

    let builder = match method {
        Method::Get => client.get(url),
        Method::Post => client.post(url),
        Method::Put => client.put(url),
        Method::Patch => client.patch(url),
        Method::Delete => client.delete(url),
        Method::Head => client.head(url),
        Method::Options => client.request(reqwest::Method::OPTIONS, url),
    };

    let builder = builder.headers(headers);
    let builder = if !body.is_empty() {
        builder.body(body.to_owned())
    } else {
        builder
    };

    match builder.send() {
        Ok(response) => {
            let status_code = response.status().as_u16();
            let status = response.status();
            let mut header_text = format!("HTTP {}\n", status);
            for (key, value) in response.headers() {
                header_text.push_str(&format!(
                    "{}: {}\n",
                    key,
                    value.to_str().unwrap_or("<binary>")
                ));
            }
            let raw_body = response
                .text()
                .unwrap_or_else(|e| format!("Error reading body: {e}"));
            let body_bytes = raw_body.len();
            let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
            let body = match serde_json::from_str::<serde_json::Value>(&raw_body) {
                Ok(json) => serde_json::to_string_pretty(&json).unwrap_or(raw_body),
                Err(_) => raw_body,
            };
            Response { headers: header_text, body, status: status_code, elapsed_ms, body_bytes }
        }
        Err(e) => {
            let mut msg = format!("Error: {e}");
            let mut source = std::error::Error::source(&e);
            while let Some(cause) = source {
                msg.push_str(&format!("\n  caused by: {cause}"));
                source = std::error::Error::source(cause);
            }
            let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
            Response { headers: String::new(), body: msg, status: 0, elapsed_ms, body_bytes: 0 }
        }
    }
}
