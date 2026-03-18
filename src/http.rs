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

pub struct Response {
    pub headers: String,
    pub body: String,
}

pub fn send_request(method: Method, url: &str, raw_headers: &str, body: &str) -> Response {
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
            let status = response.status();
            let mut header_text = format!("HTTP {}\n", status);
            for (key, value) in response.headers() {
                header_text.push_str(&format!(
                    "{}: {}\n",
                    key,
                    value.to_str().unwrap_or("<binary>")
                ));
            }
            let body = response
                .text()
                .unwrap_or_else(|e| format!("Error reading body: {e}"));
            let body = match serde_json::from_str::<serde_json::Value>(&body) {
                Ok(json) => serde_json::to_string_pretty(&json).unwrap_or(body),
                Err(_) => body,
            };
            Response {
                headers: header_text,
                body,
            }
        }
        Err(e) => {
            let mut msg = format!("Error: {e}");
            let mut source = std::error::Error::source(&e);
            while let Some(cause) = source {
                msg.push_str(&format!("\n  caused by: {cause}"));
                source = std::error::Error::source(cause);
            }
            Response {
                headers: String::new(),
                body: msg,
            }
        }
    }
}
