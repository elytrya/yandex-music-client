
use std::collections::HashMap;

use serde_json::{json, Value};

use super::cache;
use super::http;
use super::lyrics;
use super::parse::{
    album_of, credits, media_of, num, quote_from, relations_of, rich_text, sort_quotes, tags_of,
    text, verified_of,
};
use super::state;
use super::types::{GeniusQuote, GeniusSong};

const QUOTE_LIMIT: usize = 80;
const REFERENT_PAGES: usize = 6;
const REFERENT_PER_PAGE: usize = 50;
const ANNOTATION_LIMIT: usize = 48;

fn push_quote(referent: &Value, annotation: &Value, out: &mut Vec<GeniusQuote>) {
    let Some(quote) = quote_from(referent, annotation) else {
        return;
    };
    if quote.text.trim().is_empty() {
        return;
    }
    if quote.id != 0 && out.iter().any(|old| old.id == quote.id) {
        return;
    }
    out.push(quote);
}

fn collect_page(list: &[Value], out: &mut Vec<GeniusQuote>) {
    for referent in list {
        let Some(annotations) = referent.get("annotations").and_then(Value::as_array) else {
            continue;
        };
        for annotation in annotations {
            push_quote(referent, annotation, out);
        }
    }
}

async fn referents_by(token: &str, query: &str) -> (Vec<GeniusQuote>, Option<String>) {
    let mut out: Vec<GeniusQuote> = Vec::new();
    let mut failure: Option<String> = None;

    for page in 1..=REFERENT_PAGES {
        let path = format!(
            "/referents?{query}&text_format=dom,plain,html&per_page={REFERENT_PER_PAGE}&page={page}"
        );

        let body = match http::get(token, &path).await {
            Ok(body) => body,
            Err(error) => {
                if out.is_empty() {
                    failure = Some(error);
                }
                break;
            }
        };

        let Some(list) = body.get("referents").and_then(Value::as_array) else {
            break;
        };
        if list.is_empty() {
            break;
        }

        let received = list.len();
        collect_page(list, &mut out);

        if received < REFERENT_PER_PAGE {
            break;
        }
    }

    (out, failure)
}

async fn referents(token: &str, song_id: u64) -> (Vec<GeniusQuote>, Option<String>) {
    referents_by(token, &format!("song_id={song_id}")).await
}

async fn referents_web_page(token: &str, url: &str) -> (Vec<GeniusQuote>, Option<String>) {
    if url.is_empty() {
        return (Vec::new(), Some("у трека нет адреса".to_string()));
    }

    let encoded = http::encode(url);
    let path = format!(
        "/web_pages/lookup?canonical_url={encoded}&og_url={encoded}&raw_annotatable_url={encoded}"
    );

    let body = match http::get(token, &path).await {
        Ok(body) => body,
        Err(error) => return (Vec::new(), Some(error)),
    };

    let Some(page_id) = body.get("web_page").and_then(|page| num(page, "id")) else {
        return (
            Vec::new(),
            Some("Genius не знает эту страницу".to_string()),
        );
    };

    referents_by(token, &format!("web_page_id={page_id}")).await
}

async fn annotations_by_id(
    token: &str,
    marks: Vec<(u64, String)>,
) -> (Vec<GeniusQuote>, Option<String>) {
    let mut out: Vec<GeniusQuote> = Vec::new();
    let mut failure: Option<String> = None;

    for (id, fragment) in marks.into_iter().take(ANNOTATION_LIMIT) {
        let path = format!("/annotations/{id}?text_format=dom,plain,html");

        let body = match http::get(token, &path).await {
            Ok(body) => body,
            Err(error) => {
                if failure.is_none() {
                    failure = Some(error);
                }
                continue;
            }
        };

        let Some(annotation) = body.get("annotation") else {
            continue;
        };

        let referent = annotation
            .get("referent")
            .filter(|value| text(value, "fragment").is_some())
            .cloned()
            .unwrap_or_else(|| json!({ "fragment": fragment }));

        push_quote(&referent, annotation, &mut out);
    }

    (out, failure)
}

async fn referents_site(song_id: u64) -> (Vec<GeniusQuote>, Option<String>) {
    let mut out: Vec<GeniusQuote> = Vec::new();
    let mut failure: Option<String> = None;

    for page in 1..=REFERENT_PAGES {
        let path = format!(
            "/referents?song_id={song_id}&text_format=dom,plain,html&per_page={REFERENT_PER_PAGE}&page={page}"
        );

        let body = match http::get_site(&path).await {
            Ok(body) => body,
            Err(error) => {
                failure = Some(error);
                break;
            }
        };
        let Some(list) = body.get("referents").and_then(Value::as_array) else {
            break;
        };
        if list.is_empty() {
            break;
        }

        let received = list.len();
        collect_page(list, &mut out);

        if received < REFERENT_PER_PAGE {
            break;
        }
    }

    (out, failure)
}

fn collect_annotations(node: &Value, found: &mut HashMap<u64, Value>) {
    match node {
        Value::Array(list) => {
            for item in list {
                collect_annotations(item, found);
            }
        }
        Value::Object(fields) => {
            let has_body = fields
                .get("body")
                .map(|body| body.is_object() || body.is_string())
                .unwrap_or(false);

            if has_body {
                if let Some(id) = num(node, "id") {
                    found.entry(id).or_insert_with(|| node.clone());
                }
            }

            for value in fields.values() {
                collect_annotations(value, found);
            }
        }
        _ => {}
    }
}

fn walk(
    node: &Value,
    known: &HashMap<u64, Value>,
    out: &mut Vec<GeniusQuote>,
    missing: &mut Vec<(u64, String)>,
) {
    match node {
        Value::Array(list) => {
            for item in list {
                walk(item, known, out, missing);
            }
        }
        Value::Object(fields) => {
            if fields.contains_key("fragment") {
                if let Some(list) = fields.get("annotations").and_then(Value::as_array) {
                    for annotation in list {
                        push_quote(node, annotation, out);
                    }
                }

                if let Some(ids) = fields.get("annotation_ids").and_then(Value::as_array) {
                    let fragment = text(node, "fragment").unwrap_or_default();
                    for id in ids.iter().filter_map(Value::as_u64) {
                        match known.get(&id) {
                            Some(annotation) => push_quote(node, annotation, out),
                            None => {
                                if !missing.iter().any(|(old, _)| *old == id) {
                                    missing.push((id, fragment.clone()));
                                }
                            }
                        }
                    }
                }
            }

            for value in fields.values() {
                walk(value, known, out, missing);
            }
        }
        _ => {}
    }
}

fn quotes_from_page(html: &str) -> (Vec<GeniusQuote>, Vec<(u64, String)>) {
    let Some(dump) = state::preloaded(html) else {
        return (Vec::new(), Vec::new());
    };

    let mut known: HashMap<u64, Value> = HashMap::new();
    collect_annotations(&dump, &mut known);

    let mut out: Vec<GeniusQuote> = Vec::new();
    let mut missing: Vec<(u64, String)> = Vec::new();
    walk(&dump, &known, &mut out, &mut missing);
    (out, missing)
}

pub async fn build(token: &str, id: u64, refresh: bool) -> Result<GeniusSong, String> {
    if !refresh {
        if let Some(found) = cache::songs().get(&id) {
            return Ok(found);
        }
    }

    let payload = http::get(token, &format!("/songs/{id}?text_format=plain,html")).await?;
    let song = payload
        .get("song")
        .cloned()
        .ok_or_else(|| "Genius: пустой ответ по треку".to_string())?;

    let url = text(&song, "url").unwrap_or_default();

    let html = if url.is_empty() {
        Err("у трека нет страницы на Genius".to_string())
    } else {
        http::page(&url).await
    };

    let mut lyrics_error: Option<String> = None;
    let lines = match &html {
        Ok(page) => {
            let found = lyrics::from_html(page);
            if found.is_empty() {
                lyrics_error = Some("Genius не отдал текст песни".to_string());
            }
            found
        }
        Err(error) => {
            lyrics_error = Some(error.clone());
            Vec::new()
        }
    };

    let (mut quotes, api_error) = referents(token, id).await;
    let mut quotes_source = if quotes.is_empty() { "none" } else { "api" };
    let mut trace: Vec<String> = Vec::new();

    if quotes.is_empty() {
        match api_error {
            Some(error) => trace.push(format!("API: {error}")),
            None => trace.push("API: 0".to_string()),
        }
    }

    if quotes.is_empty() {
        let (found, site_error) = referents_site(id).await;
        if found.is_empty() {
            match site_error {
                Some(error) => trace.push(format!("сайт: {error}")),
                None => trace.push("сайт: 0".to_string()),
            }
        } else {
            quotes = found;
            quotes_source = "site";
        }
    }

    let mut pending: Vec<(u64, String)> = Vec::new();

    if quotes.is_empty() {
        match &html {
            Ok(page) => {
                let (found, missing) = quotes_from_page(page);
                pending = missing;
                if found.is_empty() {
                    trace.push(format!(
                        "страница: 0, ссылок на разборы: {}",
                        pending.len()
                    ));
                } else {
                    quotes = found;
                    quotes_source = "page";
                }
            }
            Err(error) => trace.push(format!("страница: {error}")),
        }
    }

    if quotes.is_empty() && !pending.is_empty() {
        let (found, id_error) = annotations_by_id(token, pending).await;
        if found.is_empty() {
            match id_error {
                Some(error) => trace.push(format!("разборы по id: {error}")),
                None => trace.push("разборы по id: 0".to_string()),
            }
        } else {
            quotes = found;
            quotes_source = "annotations";
        }
    }

    if quotes.is_empty() {
        let (found, web_error) = referents_web_page(token, &url).await;
        if found.is_empty() {
            match web_error {
                Some(error) => trace.push(format!("web_pages: {error}")),
                None => trace.push("web_pages: 0".to_string()),
            }
        } else {
            quotes = found;
            quotes_source = "web_page";
        }
    }

    sort_quotes(&mut quotes);
    quotes.truncate(QUOTE_LIMIT);

    let quotes_error = if !quotes.is_empty() {
        None
    } else if trace.is_empty() {
        Some("На Genius у этого трека пока нет разборов".to_string())
    } else {
        Some(format!(
            "Разборы не пришли — {}",
            trace.join("; ")
        ))
    };

    let primary = song.get("primary_artist").cloned().unwrap_or(Value::Null);
    let album = song.get("album").cloned().unwrap_or(Value::Null);

    let built = GeniusSong {
        id,
        title: text(&song, "title").unwrap_or_default(),
        full_title: text(&song, "full_title").unwrap_or_default(),
        url,
        artist: text(&song, "artist_names")
            .or_else(|| text(&primary, "name"))
            .unwrap_or_default(),
        artist_id: num(&primary, "id").unwrap_or(0),
        artist_url: text(&primary, "url").unwrap_or_default(),
        art: text(&song, "song_art_image_url")
            .or_else(|| text(&song, "song_art_image_thumbnail_url"))
            .or_else(|| text(&song, "header_image_url")),
        album: text(&album, "name"),
        album_url: text(&album, "url"),
        release_date: text(&song, "release_date_for_display")
            .or_else(|| text(&song, "release_date")),
        pageviews: song
            .get("stats")
            .and_then(|stats| stats.get("pageviews"))
            .and_then(Value::as_u64),
        description: rich_text(&song, "description").filter(|value| value != "?"),
        credits: credits(&song),
        lyrics: lines,
        quotes,
        album_info: album_of(&album),
        contributors: song
            .get("stats")
            .and_then(|stats| stats.get("contributors"))
            .and_then(Value::as_u64),
        concurrents: song
            .get("stats")
            .and_then(|stats| stats.get("concurrents"))
            .and_then(Value::as_u64),
        annotation_count: num(&song, "annotation_count"),
        hot: song
            .get("stats")
            .and_then(|stats| stats.get("hot"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
        recording_location: text(&song, "recording_location"),
        language: text(&song, "language"),
        apple_music_id: text(&song, "apple_music_id"),
        media: media_of(&song),
        relations: relations_of(&song),
        tags: tags_of(&song),
        verified_by: verified_of(&song),
        quotes_error,
        lyrics_error,
        quotes_source: quotes_source.to_string(),
    };

    cache::songs().put(id, built.clone());
    Ok(built)
}
