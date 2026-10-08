//! art domain.

use macroquad::prelude::*;

pub mod portrait;
pub mod profiles;
pub mod sprite;

use profiles::SURVIVOR_ART_PROFILES;
use sprite::generate_sprite;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpritePose {
    Idle,
    Moving,
    Working,
    Eating,
    Sleeping,
    Supported,
    SupportedReach,
    Tense,
    TenseGuarded,
}

impl SpritePose {
    pub const fn all() -> &'static [SpritePose] {
        &[
            SpritePose::Idle,
            SpritePose::Moving,
            SpritePose::Working,
            SpritePose::Eating,
            SpritePose::Sleeping,
            SpritePose::Supported,
            SpritePose::SupportedReach,
            SpritePose::Tense,
            SpritePose::TenseGuarded,
        ]
    }

    const fn index(self) -> usize {
        match self {
            SpritePose::Idle => 0,
            SpritePose::Moving => 1,
            SpritePose::Working => 2,
            SpritePose::Eating => 3,
            SpritePose::Sleeping => 4,
            SpritePose::Supported => 5,
            SpritePose::SupportedReach => 6,
            SpritePose::Tense => 7,
            SpritePose::TenseGuarded => 8,
        }
    }
}

pub struct PlaceholderArt {
    colonist_sprites: Vec<Texture2D>,
    production_portraits: Texture2D,
    building_atlas: Texture2D,
    crash_site_backdrop: Texture2D,
}

impl Default for PlaceholderArt {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaceholderArt {
    pub fn new() -> Self {
        let colonist_sprites = SURVIVOR_ART_PROFILES
            .iter()
            .enumerate()
            .flat_map(|(index, profile)| {
                SpritePose::all().iter().map(move |pose| {
                    texture_from_image(generate_sprite(*profile, index, *pose), FilterMode::Nearest)
                })
            })
            .collect();

        let production_portraits = Texture2D::from_file_with_format(
            include_bytes!("../../assets/art/survivor_portraits.png"),
            Some(ImageFormat::Png),
        );
        production_portraits.set_filter(FilterMode::Linear);
        let crash_site_backdrop = Texture2D::from_file_with_format(
            include_bytes!("../../assets/art/crash_site_backdrop.png"),
            Some(ImageFormat::Png),
        );
        crash_site_backdrop.set_filter(FilterMode::Linear);
        let building_atlas = Texture2D::from_file_with_format(
            include_bytes!("../../assets/art/building_atlas.png"),
            Some(ImageFormat::Png),
        );
        building_atlas.set_filter(FilterMode::Linear);

        Self {
            colonist_sprites,
            production_portraits,
            building_atlas,
            crash_site_backdrop,
        }
    }

    pub fn colonist_sprite_for_pose(
        &self,
        colonist_id: u32,
        pose: SpritePose,
    ) -> Option<&Texture2D> {
        if self.colonist_sprites.is_empty() {
            return None;
        }

        let pose_count = SpritePose::all().len();
        let profile_index = colonist_id as usize % SURVIVOR_ART_PROFILES.len();
        self.colonist_sprites
            .get(profile_index * pose_count + pose.index())
    }

    pub fn colonist_portrait(&self, colonist_id: u32) -> Option<(&Texture2D, Rect)> {
        let index = colonist_id as usize % 6;
        let column = index % 3;
        let row = index / 3;
        Some((
            &self.production_portraits,
            Rect::new(
                column as f32 * self.production_portraits.width() / 3.0,
                row as f32 * self.production_portraits.height() / 2.0,
                self.production_portraits.width() / 3.0,
                self.production_portraits.height() / 2.0,
            ),
        ))
    }

    pub fn crash_site_backdrop(&self) -> &Texture2D {
        &self.crash_site_backdrop
    }

    pub fn building_icon(&self, building_type: crate::data::building::BuildingType) -> Rect {
        let index = crate::data::building::BuildingType::all()
            .iter()
            .position(|candidate| *candidate == building_type)
            .unwrap_or(0);
        Rect::new(
            index as f32 * self.building_atlas.width() / 5.0,
            0.0,
            self.building_atlas.width() / 5.0,
            self.building_atlas.height(),
        )
    }

    pub fn building_atlas(&self) -> &Texture2D {
        &self.building_atlas
    }
}

fn texture_from_image(image: Image, filter: FilterMode) -> Texture2D {
    let texture = Texture2D::from_image(&image);
    texture.set_filter(filter);
    texture
}
