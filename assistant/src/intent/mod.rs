pub mod nlp;
pub mod schema;

use schema::Intent;

/// Classify raw user text into an Intent.
pub fn classify(text: &str) -> Intent {
    nlp::classify(text)
}
