/// App integrations — music, maps, messaging.
///
/// Uses termux-media-player for music, termux-location for maps,
/// and termux-* for messaging apps.
use crate::actions::termux_command;

// ── Music ──────────────────────────────────────────────────────

pub async fn play_music(query: &str) -> String {
    // Try termux-media-player
    let result = termux_command("termux-media-player", &["play", query]).await;
    if !result.starts_with("failed") {
        return format!("Playing: {query}");
    }

    // Fallback: try to open in default music app
    let result = termux_command("termux-open", &["--content-type", "audio/*", query]).await;

    if !result.starts_with("failed") {
        format!("Opening: {query}")
    } else {
        "I can't control music playback on this device yet.".to_string()
    }
}

pub async fn pause_music() -> String {
    let result = termux_command("termux-media-player", &["pause", ""]).await;
    if !result.starts_with("failed") {
        "Music paused".to_string()
    } else {
        "Can't pause music".to_string()
    }
}

pub async fn stop_music() -> String {
    let result = termux_command("termux-media-player", &["stop", ""]).await;
    if !result.starts_with("failed") {
        "Music stopped".to_string()
    } else {
        "Can't stop music".to_string()
    }
}

// ── Maps ───────────────────────────────────────────────────────

pub async fn open_maps(query: &str) -> String {
    let url = format!(
        "https://www.google.com/maps/search/{}",
        urlencoding::encode(query)
    );
    let result = termux_command("termux-open-url", &[&url]).await;
    if !result.starts_with("failed") {
        format!("Opening maps for: {query}")
    } else {
        "Can't open maps".to_string()
    }
}
