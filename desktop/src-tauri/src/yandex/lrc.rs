use super::dto::LyricsLineDto;

pub fn parse_lrc(raw: &str) -> Vec<LyricsLineDto> {
    let mut out = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix('[') {
            if let Some((stamp, text)) = rest.split_once(']') {
                if let Some(ms) = parse_stamp(stamp) {
                    out.push(LyricsLineDto {
                        time_ms: ms,
                        text: text.trim().to_string(),
                    });
                    continue;
                }
                if matches!(
                    stamp.split(':').next().unwrap_or("").to_ascii_lowercase().as_str(),
                    "ar" | "al" | "ti" | "au" | "by" | "re" | "ve" | "length" | "offset"
                ) {
                    continue;
                }
                out.push(LyricsLineDto {
                    time_ms: 0,
                    text: text.trim().to_string(),
                });
                continue;
            }
        }
        out.push(LyricsLineDto {
            time_ms: 0,
            text: trimmed.to_string(),
        });
    }
    if out.iter().any(|line| line.time_ms > 0) {
        out.sort_by_key(|line| line.time_ms);
    }
    out
}

pub fn parse_stamp(stamp: &str) -> Option<i64> {
    let (mm, rest) = stamp.split_once(':')?;
    let minutes: i64 = mm.trim().parse().ok()?;
    let seconds: f64 = rest.replace(',', ".").trim().parse().ok()?;
    Some(minutes * 60_000 + (seconds * 1000.0).round() as i64)
}
