use crate::{ecs::Entity, linalg::{f32, u32}};

pub struct TextureAtlas {
    pub num_textures: u32::Vec2,
    pub uv_step: f32::Vec2,
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
            }
        }
    }
}

pub struct PlayerControl { }

pub struct EnemyControl { }

pub struct Transform {
    pub position: f32::Vec2,
}

pub struct Sprite {
    // TODO: Implement a texture atlas
    pub texture_atlas: Entity,
    pub atlas_sprite_index: u32,
}