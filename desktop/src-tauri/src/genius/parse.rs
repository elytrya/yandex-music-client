
use serde_json::Value;

use super::types::{
    GeniusAlbum, GeniusAuthor, GeniusHit, GeniusMedia, GeniusPerson, GeniusPersonHit, GeniusQuote,
    GeniusRelation,
};

pub fn text(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

pub fn num(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

pub fn flag(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

pub fn decode_entities(value: &str) -> String {
    value
        .replace("&#x27;", "'")
        .replace("&#039;", "'")
        .replace("&#39;", "'")
        .replace("&quot;", "\"")
        .replace("&#34;", "\"")
        .replace("&nbsp;", " ")
        .replace("&hellip;", "…")
        .replace("&mdash;", "—")
        .replace("&ndash;", "–")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

pub fn strip_tags(html: &str) -> String {
    let normalized = html
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("<br>", "\n")
        .replace("</p>", "\n")
        .replace("</div>", "\n")
        .replace("</li>", "\n");

    let mut out = String::with_capacity(normalized.len());
    let mut inside = false;
    for ch in normalized.chars() {
        match ch {
            '<' => inside = true,
            '>' => inside = false,
            other if !inside => out.push(other),
            _ => {}
        }
    }
    decode_entities(&out)
}

pub fn tidy(lines: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in lines {
        if line.is_empty() && out.last().map(String::is_empty).unwrap_or(true) {
            continue;
        }
        out.push(line);
    }
    while out.last().map(String::is_empty).unwrap_or(false) {
        out.pop();
    }
    out
}

fn dom_text(node: &Value, out: &mut String) {
    match node {
        Value::String(value) => out.push_str(value),
        Value::Array(list) => {
            for item in list {
                dom_text(item, out);
            }
        }
        Value::Object(map) => {
            let tag = map.get("tag").and_then(Value::as_str).unwrap_or("");
            if tag == "br" {
                out.push('\n');
            }
            if let Some(children) = map.get("children") {
                dom_text(children, out);
            }
            if matches!(tag, "p" | "div" | "li" | "blockquote") {
                out.push('\n');
            }
        }
        _ => {}
    }
}

pub fn rich_text(value: &Value, key: &str) -> Option<String> {
    let node = value.get(key)?;

    if let Some(found) = node
        .get("plain")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        return Some(found.to_string());
    }

    if let Some(html) = node.get("html").and_then(Value::as_str) {
        let text = strip_tags(html);
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    if let Some(dom) = node.get("dom") {
        let mut buffer = String::new();
        dom_text(dom, &mut buffer);
        let text = tidy(buffer.lines().map(|line| line.trim().to_string()).collect()).join("\n");
        if !text.trim().is_empty() {
            return Some(text);
        }
    }

    if let Some(found) = node
        .get("markdown")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        return Some(found.to_string());
    }

    node.as_str()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

pub fn person(value: &Value, role: &str) -> Option<GeniusPerson> {
    let name = text(value, "name")?;
    Some(GeniusPerson {
        id: num(value, "id").unwrap_or(0),
        name,
        url: text(value, "url").unwrap_or_default(),
        image: text(value, "image_url"),
        role: role.to_string(),
    })
}

pub fn push_unique(out: &mut Vec<GeniusPerson>, entry: GeniusPerson) {
    if !out
        .iter()
        .any(|old| old.name == entry.name && old.role == entry.role)
    {
        out.push(entry);
    }
}

fn collect(song: &Value, key: &str, role: &str, out: &mut Vec<GeniusPerson>) {
    let Some(list) = song.get(key).and_then(Value::as_array) else {
        return;
    };
    for item in list {
        if let Some(entry) = person(item, role) {
            push_unique(out, entry);
        }
    }
}

pub fn credits(song: &Value) -> Vec<GeniusPerson> {
    let mut out: Vec<GeniusPerson> = Vec::new();

    collect(song, "producer_artists", "Продюсер", &mut out);
    collect(song, "writer_artists", "Автор", &mut out);
    collect(song, "featured_artists", "Участвует", &mut out);

    if let Some(list) = song.get("custom_performances").and_then(Value::as_array) {
        for item in list {
            let role = text(item, "label").unwrap_or_else(|| "Участник".to_string());
            collect(item, "artists", &role, &mut out);
        }
    }

    if let Some(primary) = song.get("primary_artist") {
        if let Some(entry) = person(primary, "Исполнитель") {
            push_unique(&mut out, entry);
        }
    }

    out
}

pub fn avatar_of(value: &Value) -> Option<String> {
    text(value, "image_url").or_else(|| {
        value
            .get("avatar")
            .and_then(|node| node.get("medium").or_else(|| node.get("small")))
            .and_then(|node| text(node, "url"))
    })
}

pub fn user_person(value: &Value, role: &str) -> Option<GeniusPerson> {
    let name = text(value, "name").or_else(|| text(value, "login"))?;
    Some(GeniusPerson {
        id: num(value, "id").unwrap_or(0),
        name,
        url: text(value, "url").unwrap_or_default(),
        image: avatar_of(value),
        role: role.to_string(),
    })
}

pub fn author_of(value: &Value) -> Option<GeniusAuthor> {
    let user = value.get("user").unwrap_or(value);
    let name = text(user, "name").or_else(|| text(user, "login"))?;
    Some(GeniusAuthor {
        name,
        url: text(user, "url").unwrap_or_default(),
        image: avatar_of(user),
        iq: user.get("iq").and_then(Value::as_i64),
        verified: flag(user, "is_verified") || flag(user, "is_meme_verified"),
    })
}

pub fn person_hit(value: &Value) -> Option<GeniusPersonHit> {
    let name = text(value, "name")?;
    Some(GeniusPersonHit {
        id: num(value, "id").unwrap_or(0),
        name,
        url: text(value, "url").unwrap_or_default(),
        image: text(value, "image_url").or_else(|| avatar_of(value)),
        verified: flag(value, "is_verified") || flag(value, "is_meme_verified"),
    })
}

fn contribution_label(raw: &str) -> String {
    match raw {
        "producer_artists" => "Продюсер",
        "writer_artists" => "Автор",
        "performance_artists" => "Исполнение",
        "featured_artists" => "Участвует",
        "lyrics" => "Текст",
        "annotations" => "Аннотации",
        other => return other.replace('_', " "),
    }
    .to_string()
}

pub fn verified_of(song: &Value) -> Vec<GeniusPerson> {
    let mut out: Vec<GeniusPerson> = Vec::new();

    if let Some(list) = song
        .get("verified_annotations_by")
        .and_then(Value::as_array)
    {
        for item in list {
            if let Some(entry) = user_person(item, "Проверяет аннотации") {
                push_unique(&mut out, entry);
            }
        }
    }

    if let Some(list) = song.get("verified_contributors").and_then(Value::as_array) {
        for item in list {
            let role = item
                .get("contributions")
                .and_then(Value::as_array)
                .map(|arr| {
                    arr.iter()
                        .filter_map(Value::as_str)
                        .map(contribution_label)
                        .collect::<Vec<String>>()
                        .join(", ")
                })
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "Проверенный участник".to_string());

            let entry = item
                .get("artist")
                .and_then(|node| user_person(node, &role))
                .or_else(|| item.get("user").and_then(|node| user_person(node, &role)));

            if let Some(entry) = entry {
                push_unique(&mut out, entry);
            }
        }
    }

    out
}

pub fn media_of(song: &Value) -> Vec<GeniusMedia> {
    let Some(list) = song.get("media").and_then(Value::as_array) else {
        return Vec::new();
    };
    list.iter()
        .filter_map(|item| {
            let url = text(item, "url")?;
            Some(GeniusMedia {
                provider: text(item, "provider").unwrap_or_default(),
                kind: text(item, "type").unwrap_or_default(),
                url,
            })
        })
        .collect()
}

pub fn tags_of(song: &Value) -> Vec<String> {
    song.get("tags")
        .and_then(Value::as_array)
        .map(|list| list.iter().filter_map(|item| text(item, "name")).collect())
        .unwrap_or_default()
}

pub fn album_of(album: &Value) -> Option<GeniusAlbum> {
    let name = text(album, "name")?;
    Some(GeniusAlbum {
        id: num(album, "id").unwrap_or(0),
        name,
        url: text(album, "url").unwrap_or_default(),
        art: text(album, "cover_art_url").or_else(|| text(album, "cover_art_thumbnail_url")),
        artist: album.get("artist").and_then(|node| text(node, "name")),
        release_date: text(album, "release_date_for_display")
            .or_else(|| text(album, "release_date")),
    })
}

pub fn hit_from(song: &Value) -> Option<GeniusHit> {
    let title = text(song, "title")?;
    Some(GeniusHit {
        id: num(song, "id").unwrap_or(0),
        title,
        full_title: text(song, "full_title").unwrap_or_default(),
        artist: text(song, "artist_names")
            .or_else(|| song.get("primary_artist").and_then(|node| text(node, "name")))
            .unwrap_or_default(),
        url: text(song, "url").unwrap_or_default(),
        art: text(song, "song_art_image_thumbnail_url")
            .or_else(|| text(song, "song_art_image_url"))
            .or_else(|| text(song, "header_image_thumbnail_url")),
    })
}

fn relation_label(kind: &str) -> String {
    match kind {
        "samples" => "Семплирует",
        "sampled_in" => "Семплируется в",
        "interpolates" => "Интерполирует",
        "interpolated_by" => "Интерполируется в",
        "cover_of" => "Кавер на",
        "covered_by" => "Кавер-версии",
        "remix_of" => "Ремикс на",
        "remixed_by" => "Ремиксы",
        "live_version_of" => "Живая версия",
        "performed_live_as" => "Вживую исполнялась как",
        "translation_of" => "Перевод трека",
        "translations" => "Переводы",
        other => return other.replace('_', " "),
    }
    .to_string()
}

pub fn relations_of(song: &Value) -> Vec<GeniusRelation> {
    let Some(list) = song.get("song_relationships").and_then(Value::as_array) else {
        return Vec::new();
    };

    let mut out: Vec<GeniusRelation> = Vec::new();
    for item in list {
        let songs: Vec<GeniusHit> = item
            .get("songs")
            .and_then(Value::as_array)
            .map(|arr| arr.iter().filter_map(hit_from).collect())
            .unwrap_or_default();
        if songs.is_empty() {
            continue;
        }
        let kind = text(item, "relationship_type")
            .or_else(|| text(item, "type"))
            .unwrap_or_default();
        out.push(GeniusRelation {
            kind: relation_label(&kind),
            songs,
        });
    }
    out
}

pub fn quote_from(referent: &Value, annotation: &Value) -> Option<GeniusQuote> {
    let body = rich_text(annotation, "body")?;
    let fragment = text(referent, "fragment").unwrap_or_default();

    let authors = annotation
        .get("authors")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(author_of).collect::<Vec<_>>())
        .unwrap_or_default();

    Some(GeniusQuote {
        id: num(annotation, "id").unwrap_or(0),
        fragment,
        text: body,
        url: text(annotation, "url")
            .or_else(|| text(annotation, "share_url"))
            .or_else(|| text(referent, "url"))
            .unwrap_or_default(),
        votes: annotation
            .get("votes_total")
            .and_then(Value::as_i64)
            .unwrap_or(0),
        verified: flag(annotation, "verified"),
        pinned: flag(annotation, "pinned"),
        state: text(annotation, "state").unwrap_or_default(),
        comments: annotation
            .get("comment_count")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        authors,
    })
}

pub fn sort_quotes(out: &mut [GeniusQuote]) {
    out.sort_by(|a, b| {
        b.pinned
            .cmp(&a.pinned)
            .then(b.verified.cmp(&a.verified))
            .then(b.votes.cmp(&a.votes))
    });
}
