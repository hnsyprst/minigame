use miniquad::{date, window, Bindings, BufferSource, Pipeline, RenderingBackend, UniformsSource};

use crate::{component::{PlayerControl, Sprite, Transform}, ecs::World, linalg::Vec2, shader};

pub fn movement_system(world: &mut World) {
    let mut t = date::now() * 0.3;
    for (entity, (player_control, mut transform)) in world.query_mut::<(&PlayerControl, &Transform)>() {
        println!("id: {} generation: {} x: {} y: {}", entity.get_id(), entity.get_generation(), transform.position.x, transform.position.y);
        t += entity.get_id() as f64;
        transform.position.x = t.sin() as f32 * 0.5;
        transform.position.y = (t * 3.).cos() as f32 * 0.5;
    }
}

pub fn render_system(
    world: &World,
    ctx: &mut Box<dyn RenderingBackend>,
    bindings: &Bindings,
    pipeline: &Pipeline,
) {
    let positions: Vec<Vec2> = world.query::<(&Transform, &Sprite)>()
        .map(| (entity, (transform, sprite)) | {
            transform.position
        })
        .collect();

    ctx.buffer_update(
        bindings.vertex_buffers[1],
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
