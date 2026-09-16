#[test]
fn toolkit_bundled_font_is_available() {
    assert!(macroquad_toolkit::ui::builtin_rajdhani_semibold_font_bytes().len() > 100_000);
}
