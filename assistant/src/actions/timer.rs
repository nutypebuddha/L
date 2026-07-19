use std::collections::HashMap;
use std::sync::OnceLock;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

/// Active timers: label → (expiry_instant, abort_handle).
type TimerMap = RwLock<HashMap<String, tokio::task::JoinHandle<()>>>;

static TIMERS: OnceLock<TimerMap> = OnceLock::new();

fn timers() -> &'static TimerMap {
    TIMERS.get_or_init(|| RwLock::new(HashMap::new()))
}

pub async fn set_timer(secs: u64, label: Option<&str>) -> String {
    let label = label.unwrap_or("timer").to_string();
    let display_label = label.clone();

    // Cancel existing timer with same label
    cancel_timer(Some(&label)).await;

    let handle = {
        let label = label.clone();
        tokio::spawn(async move {
            sleep(Duration::from_secs(secs)).await;
            eprintln!("[TIMER EXPIRED: {label}]");
            // TODO: play alarm sound via TTS or audio file
        })
    };

    timers().write().await.insert(label.clone(), handle);

    let mins = secs / 60;
    let remaining = secs % 60;
    if mins > 0 && remaining > 0 {
        format!("Timer set for {display_label}: {mins} minutes {remaining} seconds")
    } else if mins > 0 {
        format!("Timer set for {display_label}: {mins} minutes")
    } else {
        format!("Timer set for {display_label}: {remaining} seconds")
    }
}

pub async fn cancel_timer(label: Option<&str>) -> String {
    let timers = timers().write().await;

    if let Some(label) = label {
        if let Some(handle) = timers.get(label) {
            handle.abort();
            return format!("Cancelled timer: {label}");
        }
        return format!("No timer named '{label}' found");
    }

    // Cancel all
    let count = timers.len();
    for handle in timers.values() {
        handle.abort();
    }
    format!("Cancelled {count} timer(s)")
}
