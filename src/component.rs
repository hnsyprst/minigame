use crate::{ecs::Entity, linalg::{f32::{self, Vec2}, u32, u8}};

pub struct TextureAtlas {
    pub uv_offsets: Vec<f32::Vec2>,
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
        let uv_step = f32::Vec2 {
            x: 1.0 / num_textures.x as f32,
            y: 1.0 / num_textures.y as f32,
        };
        let mut uv_offsets = Vec::new();
        for row_idx in 0..num_textures.y {
            for col_idx in 0..num_textures.x {
                uv_offsets.push(Vec2 {
                    x: uv_step.x * col_idx as f32,
                    y: 1.0 - uv_step.y * (row_idx as f32 + 1.0),
                })
            }
        }
        TextureAtlas {
            uv_offsets,
        }
    }
}

pub struct TileMap {
    pub tiles: u8::Matrix,
    pub tile_positions: Vec<f32::Vec2>,
}

impl TileMap {
    pub fn new(
        tiles: u8::Matrix,
        tile_size: f32::Vec2,
    ) -> Self{
        let mut tile_positions = Vec::new();
        for (row_idx, row) in tiles.iter_rows().enumerate() {
            for (col_idx, _) in row.iter().enumerate() {
                tile_positions.push(f32::Vec2 {
                    x: col_idx as f32 * tile_size.x,
                    y: (tiles.height() - 1 - row_idx) as f32 * tile_size.y, // Place tiles in reverse y order
                });
            }
        }
        TileMap {
            tiles,
            tile_positions,
        }
    }
}

pub struct PlayerControl { }

pub struct EnemyControl { }

pub struct Transform {
    pub position: f32::Vec2,
}

// This is kind of a cop out from implementing proper hierarchical components (`Parent` and `Child`).
// There aren't currently any objects in the game that require more than a single layer hierarchy,
// so this will suffice (for now?)
pub struct ChildOf {
    pub parent: Entity,
}

pub struct Sprite {
    pub atlas_texture_index: usize,
}

pub struct ShootsBullet {
    pub bullet_speed: f32,
    pub is_active: bool,
}