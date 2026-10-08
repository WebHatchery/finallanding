//! Turning chronicle template keys into prose.

use crate::data::{fill_template, game_data};

/// Render a chronicle template, choosing a variant from `variant` so repeated
/// events do not read identically.
pub fn line(key: &str, variant: u64, values: &[(&str, &str)]) -> String {
    match game_data().chronicle_template(key, variant) {
        Some(template) => fill_template(template, values),
        None => {
            eprintln!("missing chronicle template '{key}'");
            key.to_owned()
        }
    }
}
