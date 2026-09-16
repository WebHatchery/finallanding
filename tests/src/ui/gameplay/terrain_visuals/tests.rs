use super::*;

#[test]
fn test_terrain_detail_is_deterministic_and_skips_missing_cells() {
    assert_eq!(
        terrain_detail(Some(CellType::Empty), 7, 11),
        terrain_detail(Some(CellType::Empty), 7, 11)
    );
    assert_eq!(terrain_detail(None, 7, 11), TerrainDetail::None);
}

#[test]
fn test_crash_site_detail_adds_deterministic_map_dressing() {
    assert_eq!(crash_site_detail(10, 10), Some(TerrainDetail::SupplyCrate));
    assert_eq!(crash_site_detail(15, 5), Some(TerrainDetail::SignalBeacon));
    assert_eq!(crash_site_detail(5, 11), Some(TerrainDetail::HullPanel));
    assert_eq!(crash_site_detail(13, 7), Some(TerrainDetail::FuelDrum));
    assert_eq!(crash_site_detail(8, 7), Some(TerrainDetail::Wreckage));
    assert_eq!(crash_site_detail(4, 4), Some(TerrainDetail::Track));
    assert_eq!(crash_site_detail(8, 6), Some(TerrainDetail::Cable));
    assert_eq!(crash_site_detail(0, 0), None);
}

#[test]
fn test_terrain_color_varies_without_leaving_palette() {
    let first = terrain_color(Some(CellType::Empty), 1, 1);
    let second = terrain_color(Some(CellType::Empty), 2, 1);

    assert_ne!(first, second);
    assert!((0.14..=0.22).contains(&first.r));
    assert!((0.08..=0.14).contains(&first.b));
}
