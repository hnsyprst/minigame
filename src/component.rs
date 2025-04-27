use crate::linalg::Vec2;

pub struct PlayerControl { }

pub struct Transform {
    pub position: Vec2,
}

pub struct Sprite {
    // TODO: Implement a texture atlas
    pub atlas_id: i8,
}