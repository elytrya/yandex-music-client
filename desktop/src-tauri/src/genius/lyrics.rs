
use super::parse::{decode_entities, tidy};

fn blocks_by(html: &str, needle: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cursor = 0usize;

    while let Some(found) = html[cursor..].find(needle) {
        let attr = cursor + found;
        let Some(tag_end) = html[attr..].find('>') else {
            break;
        };

        let start = attr + tag_end + 1;
        let mut depth = 1usize;
        let mut i = start;
        let mut end = start;

        while i < html.len() {
            let rest = &html[i..];
            if rest.starts_with("<div") {
                depth += 1;
                i += 4;
                continue;
            }
            if rest.starts_with("</div>") {
                depth -= 1;
                if depth == 0 {
                    end = i;
                    break;
                }
                i += 6;
                continue;
            }
            i += rest.chars().next().map(char::len_utf8).unwrap_or(1);
        }

        if end > start {
            out.push(html[start..end].to_string());
            cursor = end;
        } else {
            cursor = start;
        }
    }

    out
}

pub fn lyric_blocks(html: &str) -> Vec<String> {
    for needle in [
        "data-lyrics-container=\"true\"",
        "data-lyrics-container='true'",
        "class=\"lyrics\"",
        "class=\"Lyrics__Container",
    ] {
        let found = blocks_by(html, needle);
        if !found.is_empty() {
            return found;
        }
    }
    Vec::new()
}

pub fn block_to_lines(fragment: &str) -> Vec<String> {
    let normalized = fragment
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("<br>", "\n")
        .replace("</div>", "\n")
        .replace("</p>", "\n");

    let mut plain_text = String::with_capacity(normalized.len());
    let mut inside_tag = false;
    for ch in normalized.chars() {
        match ch {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            other if !inside_tag => plain_text.push(other),
            _ => {}
        }
    }

    decode_entities(&plain_text)
        .lines()
        .map(|line| line.trim().to_string())
        .collect()
}

fn is_embed_line(value: &str) -> bool {
    if value == "Embed" {
        return true;
    }
    let Some(head) = value.strip_suffix("Embed") else {
        return false;
    };
    !head.is_empty()
        && head
            .chars()
            .all(|ch| ch.is_ascii_digit() || ch == '.' || ch == ',' || ch == 'K' || ch == 'M')
}

fn is_body_chrome(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    for junk in [
        "you might also like",
        "copy link",
        "cancel",
        "how to format lyrics",
        "see all",
        "advisory",
    ] {
        if value.eq_ignore_ascii_case(junk) {
            return true;
        }
    }
    if value.starts_with("See ") && value.contains("Get tickets as low as") {
        return true;
    }
    is_embed_line(value)
}

fn is_section(value: &str) -> bool {
    value.starts_with('[') && value.ends_with(']') && value.chars().count() > 2
}

fn is_header_chrome(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }

    let mut words = value.split_whitespace();
    if let (Some(head), Some(tail)) = (words.next(), words.next()) {
        if words.next().is_none()
            && head.chars().all(|ch| ch.is_ascii_digit() || ch == ',')
            && (tail.eq_ignore_ascii_case("contributors")
                || tail.eq_ignore_ascii_case("contributor"))
        {
            return true;
        }
    }

    for junk in [
        "translations",
        "translation",
        "romanization",
        "read more",
        "lyrics",
    ] {
        if value.eq_ignore_ascii_case(junk) {
            return true;
        }
    }

    if value.ends_with("Read More") || value.ends_with(" Lyrics") {
        return true;
    }

    if is_section(value) {
        let inner: String = value
            .chars()
            .skip(1)
            .take(value.chars().count() - 2)
            .collect();
        let inner = inner.trim();
        let lower = inner.to_lowercase();
        if lower.starts_with("текст песни")
            || lower.starts_with("текст пісні")
            || lower.ends_with("lyrics")
        {
            return true;
        }
    }

    is_body_chrome(value)
}

fn cut_glued_header(value: &str) -> Option<String> {
    let marker = "Read More";
    let at = value.rfind(marker)?;
    let tail = value[at + marker.len()..].trim();
    if tail.is_empty() {
        return Some(String::new());
    }
    Some(tail.to_string())
}

pub fn strip_chrome(lines: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in lines {
        let value = line.trim();
        if is_body_chrome(value) {
            continue;
        }
        out.push(value.to_string());
    }

    let scan = out.len().min(40);
    let mut cut = 0usize;
    let mut glued: Option<(usize, String)> = None;

    for index in 0..scan {
        let value = out[index].clone();
        if value.is_empty() {
            continue;
        }
        if is_header_chrome(&value) {
            cut = index + 1;
            continue;
        }
        if value.contains("Read More") {
            if let Some(tail) = cut_glued_header(&value) {
                glued = Some((index, tail));
                cut = index;
                break;
            }
        }
        if is_section(&value) {
            break;
        }
    }

    if let Some((index, tail)) = glued {
        if tail.is_empty() {
            out[index] = String::new();
            cut = index + 1;
        } else {
            out[index] = tail;
        }
    }

    if cut > 0 && cut <= out.len() {
        out.drain(0..cut);
    }

    tidy(out)
}

pub fn from_html(html: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for block in lyric_blocks(html) {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.extend(block_to_lines(&block));
    }
    strip_chrome(lines)
}
