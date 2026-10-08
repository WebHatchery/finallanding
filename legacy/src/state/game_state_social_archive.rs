//! game state social archive domain.

use super::*;

pub fn write_social_archive_markdown(history: &[SocialHistoryEntry]) -> Result<PathBuf, String> {
    let markdown = social_archive_markdown(history);

    #[cfg(target_arch = "wasm32")]
    {
        return match crate::browser_clipboard::copy_text(&markdown) {
            crate::browser_clipboard::ClipboardCopy::Copied => {
                Ok(PathBuf::from("browser clipboard (Markdown copied)"))
            }
            crate::browser_clipboard::ClipboardCopy::Requested => {
                Ok(PathBuf::from("browser clipboard (copy requested)"))
            }
            crate::browser_clipboard::ClipboardCopy::Failed => Err(
                "Browser blocked the Markdown export. Use the copy/download panel to save it."
                    .to_owned(),
            ),
        };
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        write_native_archive(&markdown)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn write_native_archive(markdown: &str) -> Result<PathBuf, String> {
    let output_dir = PathBuf::from("docs").join("exports");
    std::fs::create_dir_all(&output_dir)
        .map_err(|error| format!("Could not create {}: {}", output_dir.display(), error))?;
    let output_path = output_dir.join("social_archive.md");
    std::fs::write(&output_path, markdown)
        .map_err(|error| format!("Could not write {}: {}", output_path.display(), error))?;
    Ok(output_path)
}

pub fn social_archive_markdown(history: &[SocialHistoryEntry]) -> String {
    let mut output = String::from("# The Final Landing Social Archive\n\n");
    output.push_str(&format!("Reports: {}\n\n", history.len()));

    for entry in history.iter().rev() {
        output.push_str(&format!("## Day {}: {}\n\n", entry.day, entry.title));
        output.push_str(&format!(
            "- Mood: {:.0}\n- Relationship: {:+.0}\n- Close pairs: {}\n- Strained pairs: {}\n\n",
            entry.average_mood, entry.average_relationship, entry.close_pairs, entry.strained_pairs
        ));
        output.push_str(&format!("{}\n\n", entry.detail));
        output.push_str(&format!("Recommendation: {}\n\n", entry.recommendation));
    }

    output
}
