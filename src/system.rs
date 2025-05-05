use std::collections::HashSet;

use miniquad::{date, window, Bindings, BufferSource, KeyCode, Pipeline, RenderingBackend, UniformsSource};

use crate::{bundle::BulletBundle, component::{Bullet, ChildOf, Collider, CollisionEvent, Enemy, Player, ShootsBullet, Sprite, TextureAtlas, TileMap, Transform, Velocity}, ecs::{Entity, World}, linalg::{f32, Vector}, shader};

pub fn player_movement_system(
    world: &mut World,
    pressed_keys: &HashSet<KeyCode>,
) {
    let mut movement_vec = f32::Vec2 { x: 0.0, y: 0.0 };
    let speed = 0.01;

    if pressed_keys.contains(&KeyCode::W) {
        movement_vec.y += 1.0;
    };
    if pressed_keys.contains(&KeyCode::A) {
        movement_vec.x -= 1.0;
    };
    if pressed_keys.contains(&KeyCode::S) {
        movement_vec.y -= 1.0;
    };
    if pressed_keys.contains(&KeyCode::D) {
        movement_vec.x += 1.0;
    };

    movement_vec = movement_vec.normalize();
    movement_vec *= speed;

    for (_, (_player_control, mut transform)) in world.query_mut::<(&Player, &Transform)>() {
        transform.position.x += movement_vec.x;
        transform.position.y += movement_vec.y;
    }
}

pub fn screen_to_world(
    screen_vec: &f32::Vec2,
) -> f32::Vec2 {
    let (screen_width, screen_height) = miniquad::window::screen_size();
    f32::Vec2 {
        x: (screen_vec.x - screen_width / 2.0) / screen_width * 2.0,
        y: (screen_height / 2.0 - screen_vec.y) / screen_height * 2.0,
    }
}

pub fn shoot_gun_system(
    world: &mut World,
    mouse_position: &f32::Vec2,
    pressed_keys: &HashSet<KeyCode>,
) {
    // TODO: Change to mouse click
    if pressed_keys.contains(&KeyCode::Space) {
        let shoot_data: Vec<(f32::Vec2, f32::Vec2)> = world.query::<(&ShootsBullet, &Transform)>()
            .map(| (entity, (shoots_bullet, transform)) | {
                let world_position = compute_world_position(world, &entity, &transform);
                let velocity_vec = (screen_to_world(mouse_position) - world_position).normalize() * shoots_bullet.bullet_speed;
                (velocity_vec, world_position)
            })
            .collect();

        for (velocity_vec, position) in shoot_data {
            let bullet = world.create_entity();
            world.add_bundle(&bullet, BulletBundle {
                transform: Transform {
                    position,
                },
                velocity: Velocity {
                    vec: velocity_vec,
                },
                ..Default::default()
            });
        }
    }
}

pub fn enemy_movement_system(
    world: &mut World,
) {
    let mut t = date::now() * 0.3;
    for (entity, (_enemy_control, mut transform)) in world.query_mut::<(&Enemy, &Transform)>() {
        t += entity.get_id() as f64;
        transform.position.x = t.sin() as f32 * 0.5;
        transform.position.y = (t * 3.).cos() as f32 * 0.5;
    }
}

pub fn apply_velocity_system(
    world: &mut World,
) {
    for (_, (velocity, mut transform)) in world.query_mut::<(&Velocity, &Transform)>() {
        transform.position += velocity.vec;
    }
}

fn compute_world_position(
    world: &World,
    entity: &Entity,
    transform: &Transform,
) -> f32::Vec2 {
    if let Some(child_of) = world.get_component::<ChildOf>(entity).unwrap() {
        let parent_transform = world.get_component::<Transform>(&child_of.parent).unwrap().expect("Parent referenced in ChildOf component did not have a Transform component!");
        transform.position + parent_transform.position
    } else {
        transform.position
    }
}

pub fn render_system(
    world: &World,
    ctx: &mut Box<dyn RenderingBackend>,
    bindings: &Bindings,
    pipeline: &Pipeline,
    texture_atlas_entity: &Entity,
) {
    let screen_size = {
        let (x, y) = window::screen_size();
        f32::Vec2 { x, y }
    };

    let mut positions = Vec::new();
    let mut uv_offsets: Vec<f32::Vec2> = Vec::new();
    let texture_atlas = world.get_component::<TextureAtlas>(texture_atlas_entity)
        .unwrap()
        .expect("Texture atlas entity missing a texture atlas component!");

    world.query::<(&Transform, &TileMap)>()
        .for_each(| (_, (transform, tile_map)) | {
            positions.extend(tile_map.tile_positions
                .iter()
                .map(| position | {
                    *position + transform.position
                })
            );
            uv_offsets.extend(
                tile_map.tiles
                    .iter()
                    .map(| atlas_texture_index | {
                        texture_atlas.uv_offsets
                            .get(*atlas_texture_index as usize)
                            .unwrap_or(&f32::Vec2 { x: 0., y: 0. })  // Use default texture on lookup error
                    })
            );
        });

    
    world.query::<(&Transform, &Sprite)>()
        .for_each(| (entity, (transform, sprite)) | {
            positions.push(compute_world_position(world, &entity, &transform));
            // TODO: Parameterise default texture
            uv_offsets.push(
                texture_atlas.uv_offsets
                    .get(sprite.atlas_texture_index)
                    .unwrap_or(&f32::Vec2 { x: 0., y: 0. })  // Use default texture on lookup error
                    .to_owned()
                );
        });

    ctx.buffer_update(
        bindings.vertex_buffers[1],
        BufferSource::slice(&uv_offsets[..]),
    );

    ctx.buffer_update(
        bindings.vertex_buffers[2],
        BufferSource::slice(&positions[..]),
    );

    ctx.begin_default_pass(Default::default());

    // Enforce aspect ratio
    // TODO: Only bother doing this when the screen size changes
    let aspect_ratio: f32 = 4.0 / 3.0;
    let (viewport_height, viewport_width, pillarbox_height, pillarbox_width) = if screen_size.y * aspect_ratio >= screen_size.x {
        (screen_size.x / aspect_ratio, screen_size.x, (screen_size.y - screen_size.x / aspect_ratio) / 2.0, 0.0)
    } else {
        (screen_size.y, screen_size.y * aspect_ratio, 0.0, (screen_size.x - screen_size.y * aspect_ratio) / 2.0)
    };
    ctx.apply_viewport(
        pillarbox_width.floor() as i32, pillarbox_height.floor() as i32, viewport_width.floor() as i32, viewport_height.floor() as i32
    );

    ctx.apply_pipeline(pipeline);
    ctx.apply_bindings(bindings);

    ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms {
        offset: (0.0, 0.0),
    }));

    ctx.draw(0, 6, positions.len() as i32);
}

fn colliders_colliding(
    world: &World,
    entity_a: &Entity,
    collider_a: &Collider,
    transform_a: &Transform,
    entity_b: &Entity,
    collider_b: &Collider,
    transform_b: &Transform,
) -> bool {
    let position_a = compute_world_position(world, entity_a, transform_a);
    let position_b = compute_world_position(world, entity_b, transform_b);
    {
        position_a.x < position_b.x + collider_b.size.x &&
        position_a.x + collider_a.size.x > position_b.x &&
        position_a.y < position_b.y + collider_b.size.y &&
        position_a.y + collider_a.size.y > position_b.y
    }
}

pub fn collision_detection_system(
    world: &mut World,
) {
    // TODO: Implement quadtree
    for (entity_a, (collider_a, transform_a)) in world.query::<(&Collider, &Transform)>() {
        for (entity_b, (collider_b, transform_b)) in world.query::<(&Collider, &Transform)>() {
            if entity_a == entity_b { continue }
            if colliders_colliding(world, &entity_a, &collider_a, &transform_a, &entity_b, &collider_b, &transform_b) && !collider_a.is_static {
                world.add_component(&entity_a, CollisionEvent { entity_a, entity_b }).unwrap();
                // We'll add the CollisionEvent component to `entity_b` on the second pass
                // FIXME: Iterating through all the entities twice is so inefficient!
            }
        }
    }
}

pub fn collision_resolution_system(
    world: &mut World,
) {
    let mut bullet_entities = Vec::new();
    for (entity, collision_event, ) in world.query::<&CollisionEvent>() {
        if world.get_component::<Bullet>(&collision_event.entity_a).unwrap().is_some() &&
        world.get_component::<Bullet>(&collision_event.entity_b).unwrap().is_none() && 
        // For now, bullets won't collide with the player
        world.get_component::<Player>(&collision_event.entity_b).unwrap().is_none()
        {
            bullet_entities.push(entity);
        }
    }
    for entity in bullet_entities {
        world.destroy_entity(entity);
    }
}

pub fn collision_cleanup_system(
    world: &mut World,
) {
    let entities: Vec<Entity> = world.query::<&CollisionEvent>()
        .map(| (entity, _) | {
            entity
        })
        .collect();
    if entities.is_empty() { return }
    for entity in entities {
        world.remove_component::<CollisionEvent>(&entity).unwrap();
    }
}