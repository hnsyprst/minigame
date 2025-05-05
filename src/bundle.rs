use crate::{component::{self, Sprite, Transform, Velocity}, ecs::{Entity, World}, linalg::f32};

// TODO: It would be great to have a macro automatically derive `add_components`!
pub trait Bundle {
    fn add_components(self, world: &mut World, entity: &Entity);
}

pub struct BulletBundle {
    pub transform: component::Transform,
    pub velocity: component::Velocity,
    pub sprite: component::Sprite,
}

impl Bundle for BulletBundle {
    fn add_components(self, world: &mut World, entity: &Entity) {
        world.add_component(entity, self.transform).unwrap();
        world.add_component(entity, self.velocity).unwrap();
        world.add_component(entity, self.sprite).unwrap();
    }
}

impl Default for BulletBundle {
    fn default() -> Self {
        BulletBundle { 
            transform: Transform {
                position: f32::Vec2 { x: 0.0, y: 0.0 },
            },
            velocity: Velocity {
                vec: f32::Vec2 { x: 0.0, y: 0.0 },
            },
            sprite: Sprite {
                atlas_texture_index: 30,
            },
        }
    }
}