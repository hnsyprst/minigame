use miniquad::date;

use crate::{component::{PlayerControl, Transform}, ecs::World};

pub fn movement_system(world: &mut World) {
    let t = date::now() * 0.3;
    for (entity, (player_control, transform)) in world.query::<(&PlayerControl, &Transform)>() {
        println!("id: {} generation: {} x: {} y: {}", entity.get_id(), entity.get_generation(), transform.position.x, transform.position.y);
    }
}


