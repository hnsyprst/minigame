use std::collections::HashSet;

use miniquad::{date, window, Bindings, BufferSource, KeyCode, Pipeline, RenderingBackend, UniformsSource};

use crate::{component::{EnemyControl, PlayerControl, Sprite, TextureAtlas, Transform}, ecs::World, linalg::{f32, Vector}, shader};

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

    for (entity, (_player_control, mut transform)) in world.query_mut::<(&PlayerControl, &Transform)>() {
        transform.position.x += movement_vec.x;
        transform.position.y += movement_vec.y;
    }
}

pub fn enemy_movement_system(
    world: &mut World,
) {
    let mut t = date::now() * 0.3;
    for (entity, (_enemy_control, mut transform)) in world.query_mut::<(&EnemyControl, &Transform)>() {
        t += entity.get_id() as f64;
        transform.position.x = t.sin() as f32 * 0.5;
        transform.position.y = (t * 3.).cos() as f32 * 0.5;
    }
}

pub fn sprite_lookup_system(
    world: &World,
    sprite: &Sprite,
) -> Option<f32::Vec2> {
    let atlas = world.get_component_mut::<TextureAtlas>(&sprite.texture_atlas).ok()??;
    // If sprite is out of bounds, return None
    let atlas_x = sprite.atlas_sprite_index % atlas.num_textures.x;
    let atlas_y = sprite.atlas_sprite_index / atlas.num_textures.x;
    if atlas_x >= atlas.num_textures.x || atlas_y >= atlas.num_textures.y {
        return None;
    }
    Some(f32::Vec2 {
        x: atlas.uv_step.x * atlas_x as f32,
        y: 1.0 - atlas.uv_step.y * (atlas_y as f32 + 1.0), // Flip the y-coordinate to index from top left
    })
}

pub fn render_system(
    world: &World,
    ctx: &mut Box<dyn RenderingBackend>,
    bindings: &Bindings,
    pipeline: &Pipeline,
) {
    let (positions, uv_offsets): (Vec<f32::Vec2>, Vec<f32::Vec2>) = world.query::<(&Transform, &Sprite)>()
        .map(| (_, (transform, sprite)) | {
            (
                transform.position,
                // TODO: Would be better to get all sprites with the same atlas and run the lookup over all of them at once
                // rather than looking up the texture atlas over and over again
                // maybe even just lookup UVs once when the Sprite component is added
                sprite_lookup_system(world, &sprite).unwrap_or(f32::Vec2 { x: 0., y: 0. }) // Use default texture on lookup error
            )
        })
        .collect();

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
    let (screen_width, screen_height) = window::screen_size();
    let (viewport_height, viewport_width, pillarbox_height, pillarbox_width) = if screen_height * aspect_ratio >= screen_width {
        (screen_width / aspect_ratio, screen_width, (screen_height - screen_width / aspect_ratio) / 2.0, 0.0)
    } else {
        (screen_height, screen_height * aspect_ratio, 0.0, (screen_width - screen_height * aspect_ratio) / 2.0)
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
