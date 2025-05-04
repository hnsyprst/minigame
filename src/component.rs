use crate::{ecs::Entity, linalg::{f32, u32, u8}};

pub struct TextureAtlas {
    pub num_textures: u32::Vec2,
    pub uv_step: f32::Vec2,
    pub texture_size: u32::Vec2,
}

impl TextureAtlas {
    pub fn new(
        atlas_size: u32::Vec2,
        texture_size: u32::Vec2,
    ) -> Self{
        let num_textures = u32::Vec2 {
            x: (atlas_size.x / texture_size.x),
            y: (atlas_size.y / texture_size.y),
        };
        TextureAtlas {
            num_textures,
            uv_step: f32::Vec2 {
                x: 1.0 / num_textures.x as f32,
                y: 1.0 / num_textures.y as f32,
            },
            texture_size,
        }
    }
}

// TODO: Should this just have a new() method and immediately get `tiles_atlas_uv_offsets` 
// and `tiles_positions` from the texture atlas instead of requiring a whole system?
// `tiles` is useless after init()
pub struct TileMap {
    pub texture_atlas: Entity,
    pub tiles: u8::Matrix,
    pub tiles_atlas_uv_offsets: Option<Vec<f32::Vec2>>,
    pub tiles_positions: Option<Vec<f32::Vec2>>,
}

pub struct PlayerControl { }

pub struct EnemyControl { }

pub struct Transform {
    pub position: f32::Vec2,
}

// TODO: Should this just have a new() method and immediately get `atlas_uv_offset` from
// the texture atlas instead of requiring a whole system? `atlas_sprite_index` is useless after init()
pub struct Sprite {
    pub texture_atlas: Entity,
    pub atlas_sprite_index: u32,
    pub atlas_uv_offset: Option<f32::Vec2>,
}