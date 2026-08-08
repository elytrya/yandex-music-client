
use serde_json::Value;

fn find_unescaped_quote(value: &str) -> Option<usize> {
    let bytes = value.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index += 2,
            b'\'' => return Some(index),
            _ => index += 1,
        }
    }
    None
}

fn unescape_js(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('\'') => out.push('\''),
            Some('"') => out.push_str("\\\""),
            Some('\\') => out.push_str("\\\\"),
            Some('/') => out.push('/'),
            Some('n') => out.push_str("\\n"),
            Some('r') => out.push_str("\\r"),
            Some('t') => out.push_str("\\t"),
            Some('b') => out.push_str("\\b"),
            Some('f') => out.push_str("\\f"),
            Some('u') => {
                out.push_str("\\u");
                for _ in 0..4 {
                    if let Some(digit) = chars.next() {
                        out.push(digit);
                    }
                }
            }
            Some(other) => out.push(other),
            None => break,
        }
    }

    out
}

pub fn preloaded(html: &str) -> Option<Value> {
    let anchor = html.find("__PRELOADED_STATE__")?;
    let rest = &html[anchor..];

    if let Some(at) = rest.find("JSON.parse('") {
        let tail = &rest[at + "JSON.parse('".len()..];
        let end = find_unescaped_quote(tail)?;
        let raw = unescape_js(&tail[..end]);
        if let Ok(parsed) = serde_json::from_str::<Value>(&raw) {
            return Some(parsed);
        }
    }

    let at = rest.find('=')?;
    let tail = rest[at + 1..].trim_start();
    if !tail.starts_with('{') {
        return None;
    }

    let bytes = tail.as_bytes();
    let mut depth = 0usize;
    let mut inside_string = false;
    let mut escaped = false;

    for (index, byte) in bytes.iter().enumerate() {
        if inside_string {
            if escaped {
                escaped = false;
            } else if *byte == b'\\' {
                escaped = true;
            } else if *byte == b'"' {
                inside_string = false;
            }
            continue;
        }
        match byte {
            b'"' => inside_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return serde_json::from_str::<Value>(&tail[..=index]).ok();
                }
            }
            _ => {}
        }
    }

    None
}
