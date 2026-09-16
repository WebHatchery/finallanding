//! game state social archive domain.

use super::*;

pub fn write_social_archive_markdown(history: &[SocialHistoryEntry]) -> Result<PathBuf, String> {
    let markdown = social_archive_markdown(history);

    #[cfg(target_arch = "wasm32")]
    {
        browser_download::download_text("final-landing-social-archive.md", &markdown)?;
        return Ok(PathBuf::from(
            "browser download: final-landing-social-archive.md",
        ));
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

#[cfg(target_arch = "wasm32")]
mod browser_download {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;
    use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};

    pub fn download_text(filename: &str, content: &str) -> Result<(), String> {
        let window = web_sys::window().ok_or_else(|| "Browser window unavailable".to_owned())?;
        let document = window
            .document()
            .ok_or_else(|| "Browser document unavailable".to_owned())?;
        let parts = js_sys::Array::new();
        parts.push(&JsValue::from_str(content));
        let options = BlobPropertyBag::new();
        options.set_type("text/markdown;charset=utf-8");
        let blob = Blob::new_with_str_sequence_and_options(&parts, &options)
            .map_err(|error| format!("Could not prepare archive download: {error:?}"))?;
        let url = Url::create_object_url_with_blob(&blob)
            .map_err(|error| format!("Could not create archive download URL: {error:?}"))?;
        let anchor = document
            .create_element("a")
            .map_err(|error| format!("Could not create archive download link: {error:?}"))?
            .dyn_into::<HtmlAnchorElement>()
            .map_err(|_| "Could not prepare archive download link".to_owned())?;
        anchor.set_href(&url);
        anchor.set_download(filename);
        anchor
            .style()
            .set_property("display", "none")
            .map_err(|error| format!("Could not hide archive download link: {error:?}"))?;
        let body = document
            .body()
            .ok_or_else(|| "Browser document has no body".to_owned())?;
        body.append_child(&anchor)
            .map_err(|error| format!("Could not attach archive download link: {error:?}"))?;
        anchor.click();
        body.remove_child(&anchor)
            .map_err(|error| format!("Could not remove archive download link: {error:?}"))?;
        Url::revoke_object_url(&url)
            .map_err(|error| format!("Could not release archive download URL: {error:?}"))?;
        Ok(())
    }
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
