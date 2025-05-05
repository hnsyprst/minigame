use crate::{component::{self, Bullet, Collider, Sprite, Transform, Velocity}, ecs::{Entity, World}, linalg::f32};

// TODO: It would be great to have a macro automatically derive `add_components`!
pub trait Bundle {
    fn add_components(self, world: &mut World, entity: &Entity);
}

pub struct BulletBundle {
    pub bullet: component::Bullet,
    pub transform: component::Transform,
    pub velocity: component::Velocity,
    pub sprite: component::Sprite,
    pub collider: component::Collider,
}

impl Bundle for BulletBundle {
    fn add_components(self, world: &mut World, entity: &Entity) {
        world.add_component(entity, self.bullet).unwrap();
        world.add_component(entity, self.transform).unwrap();
        world.add_component(entity, self.velocity).unwrap();
        world.add_component(entity, self.sprite).unwrap();
        world.add_component(entity, self.collider).unwrap();
    }
}

impl Default for BulletBundle {
    fn default() -> Self {
        BulletBundle { 
            bullet: Bullet { },
            transform: Transform {
                position: f32::Vec2 { x: 0.0, y: 0.0 },
            },
            velocity: Velocity {
                vec: f32::Vec2 { x: 0.0, y: 0.0 },
            },
            sprite: Sprite {
                atlas_texture_index: 30,
            },
            collider: Collider {
                size: f32::Vec2 { x: 0.1, y: 0.1 }, // FIXME: Remove magic number (quad size from main.rs)
                is_static: false,
            },
        }
    }
}