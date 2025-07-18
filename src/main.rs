mod ecs;
mod resources;
mod linalg;
mod component;
mod system;
mod bundle;

use std::{collections::HashSet};

use miniquad::*;
use resources::ResourceManager;
use linalg::{f32, u32};
use system::{apply_velocity_system, collision_cleanup_system, collision_detection_system, collision_resolution_system, enemy_movement_system, player_movement_system, render_system, shoot_gun_system};

const MAX_SPRITES: usize = 1024;

#[repr(C)]
struct Vertex {
    pos: f32::Vec2,
    uv: f32::Vec2,
}

struct Stage {
    ctx: Box<dyn RenderingBackend>,
    pipeline: Pipeline,
    bindings: Bindings,
    mouse_position: f32::Vec2,
    pressed_keys: HashSet<KeyCode>,
    world: ecs::World,
    texture_atlas_entity: ecs::Entity, // TODO: Update the resource manager so this isn't an entity anymore
}

impl Stage {
    pub fn new() -> Stage {
        let mut ctx: Box<dyn RenderingBackend> = window::new_rendering_backend();

        #[rustfmt::skip]
        let vertices: [Vertex; 4] = [
            Vertex { pos : f32::Vec2 { x: -0.05, y: -0.05 }, uv: f32::Vec2 { x: 0., y: 0. } },
            Vertex { pos : f32::Vec2 { x:  0.05, y: -0.05 }, uv: f32::Vec2 { x: 0.125, y: 0. } },
            Vertex { pos : f32::Vec2 { x:  0.05, y:  0.05 }, uv: f32::Vec2 { x: 0.125, y: 0.125 } },
            Vertex { pos : f32::Vec2 { x: -0.05, y:  0.05 }, uv: f32::Vec2 { x: 0., y: 0.125 } },
        ];
        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices),
        );

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices),
        );

        let instance_uv_offsets_buffer = ctx.new_buffer(
            BufferType::VertexBuffer, 
            BufferUsage::Stream, 
            BufferSource::empty::<f32::Vec2>(MAX_SPRITES),
        );

        let instance_positions_buffer = ctx.new_buffer(
            BufferType::VertexBuffer, 
            BufferUsage::Stream, 
            BufferSource::empty::<f32::Vec2>(MAX_SPRITES),
        );

        // Load necessary resources
        let mut resource_manager = ResourceManager::new();
        let tilemap_resource = resource_manager.register_resource("src/map_1.csv");
        let texture_atlas_resource = resource_manager.register_resource("src/atlas.png");
        resource_manager.load_resources().unwrap();

        // Load tilemap
        let tiles = resource_manager.get_as_tiles(&tilemap_resource).unwrap();
        
        // Load player texture
        let texture_atlas_size = u32::Vec2 { x: 128, y: 128 };
        let sprite_size = u32::Vec2 { x: 16, y: 16 };
        let pixels = resource_manager.get_as_rgba8(
            &texture_atlas_resource,
            &texture_atlas_size,
        ).unwrap();
        let texture = ctx.new_texture_from_data_and_format(
            &pixels,
            TextureParams {
                kind: TextureKind::Texture2D,
                format: TextureFormat::RGBA8,
                wrap: TextureWrap::Clamp,
                min_filter: FilterMode::Nearest,
                mag_filter: FilterMode::Nearest,
                mipmap_filter: MipmapFilterMode::None,
                width: texture_atlas_size.x,
                height: texture_atlas_size.y,
                allocate_mipmaps: false,
                sample_count: 1,
            },
        );
        
        // Bind to GPU
        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer, instance_uv_offsets_buffer, instance_positions_buffer],
            index_buffer,
            images: vec![texture],
        };

        // Load shader
        let shader = ctx
            .new_shader(
                match ctx.info().backend {
                    Backend::OpenGl => ShaderSource::Glsl {
                        vertex: shader::VERTEX,
                        fragment: shader::FRAGMENT,
                    },
                    Backend::Metal => ShaderSource::Msl {
                        program: shader::METAL,
                    },
                },
                shader::meta(),
            )
            .unwrap();
        
        // Define pipeline
        let pipeline = ctx.new_pipeline(
            &[
                BufferLayout::default(),
                BufferLayout {
                    step_func: VertexStep::PerInstance,
                    ..Default::default()
                },
                BufferLayout {
                    step_func: VertexStep::PerInstance,
                    ..Default::default()
                },
            ],
            &[
                VertexAttribute::with_buffer("in_pos", VertexFormat::Float2, 0),
                VertexAttribute::with_buffer("in_uv", VertexFormat::Float2, 0),
                VertexAttribute::with_buffer("in_instance_uv_offset", VertexFormat::Float2, 1),
                VertexAttribute::with_buffer("in_instance_pos", VertexFormat::Float2, 2),
            ],
            shader,
            PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha))
                ),
                ..Default::default()
            }
        );
        
        // Create HashSet for storing pressed keys
        let pressed_keys = HashSet::new();
        // Create Vec2 for storing mouse position
        let mouse_position = f32::Vec2 { x: 0.0, y: 0.0 };

        // Set up level
        let mut world = ecs::World::new();
        world.register_component::<component::Transform>();
        world.register_component::<component::Velocity>();
        world.register_component::<component::Sprite>();
        world.register_component::<component::Player>();
        world.register_component::<component::Enemy>();
        world.register_component::<component::Bullet>();
        world.register_component::<component::Wall>();
        world.register_component::<component::TextureAtlas>();
        world.register_component::<component::TileMap>();
        world.register_component::<component::ChildOf>();
        world.register_component::<component::ShootsBullet>();
        world.register_component::<component::Collider>();
        world.register_component::<component::CollisionEvent>();

        // Create texture atlas
        // TODO: Explicitly link this to `texture`
        let texture_atlas_entity = world.create_entity();
        world.add_component(&texture_atlas_entity, component::TextureAtlas::new(texture_atlas_size, sprite_size)).unwrap();

        // Create tile map
        let tile_map = world.create_entity();
        world.add_component(&tile_map, component::Transform { position: f32::Vec2 { x: -1.0, y: -1.0 } }).unwrap();
        let tile_size = f32::Vec2 { x: 0.1, y: 0.1 };
        let tile_map_component = component::TileMap::new(tiles, tile_size);
        let collidable_tile_ids: HashSet<u8> = [0, 1, 2, 3, 4, 5, 8, 9, 10, 11, 12, 13, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27]
            .into_iter()
            .collect();
        tile_map_component.spawn_colliders(&mut world, &tile_map, tile_size, &collidable_tile_ids);
        world.add_component(&tile_map, tile_map_component).unwrap();

        // Create player
        let player = world.create_entity();
        world.add_component(&player, component::Transform { position: f32::Vec2 { x: 0.5, y: 0.2 } }).unwrap();
        world.add_component(&player, component::Sprite { atlas_texture_index: 28 }).unwrap();
        world.add_component(&player, component::Player { } ).unwrap();
        world.add_component(&player, component::Collider { size: f32::Vec2 { x: 0.1, y: 0.1 }, is_static: false } ).unwrap();

        // Create gun to demonstrate ChildOf component
        let gun = world.create_entity();
        world.add_component(&gun, component::ChildOf { parent: player }).unwrap();
        world.add_component(&gun, component::Transform { position: f32::Vec2 { x: 0.05, y: 0.0 } }).unwrap();
        world.add_component(&gun, component::Sprite { atlas_texture_index: 29 }).unwrap();
        world.add_component(&gun, component::ShootsBullet { bullet_speed: 0.01, is_active: true }).unwrap();

        // Create enemy
        let enemy = world.create_entity();
        world.add_component(&enemy, component::Transform { position: f32::Vec2 { x: 0.5, y: 0.7 } }).unwrap();
        world.add_component(&enemy, component::Sprite { atlas_texture_index: 36 }).unwrap();
        world.add_component(&enemy, component::Enemy { } ).unwrap();

        Stage {
            ctx,
            pipeline,
            bindings,
            pressed_keys,
            mouse_position,
            world,
            texture_atlas_entity,
        }
    }
}

impl EventHandler for Stage {
    fn update(&mut self) {
        player_movement_system(
            &mut self.world,
            &self.pressed_keys,
        );
        enemy_movement_system(
            &mut self.world,
        );
        shoot_gun_system(
            &mut self.world,
            &self.mouse_position,
            &self.pressed_keys,
        );
        apply_velocity_system(
            &mut self.world,
        );
        collision_detection_system(
            &mut self.world,
        );
        collision_resolution_system(
            &mut self.world,
        );
        collision_cleanup_system(
            &mut self.world,
        );
    }

    fn draw(&mut self) {
        render_system(
            &self.world,
            &mut self.ctx,
            &self.bindings,
            &self.pipeline,
            &self.texture_atlas_entity,
        );
        self.ctx.end_render_pass();
        self.ctx.commit_frame();
    }

    fn key_down_event(
        &mut self,
        _keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        self.pressed_keys.insert(_keycode);
    }

    fn key_up_event(
        &mut self,
        _keycode: KeyCode,
        _keymods: KeyMods,
    ) {
        self.pressed_keys.remove(&_keycode);
    }

    fn mouse_motion_event(
        &mut self,
        _x: f32,
        _y: f32,
    ) {
        self.mouse_position.x = _x;
        self.mouse_position.y = _y;
    }
}

fn main() {
    let mut conf = conf::Conf::default();
    let metal = std::env::args().nth(1).as_deref() == Some("metal");
    conf.platform.apple_gfx_api = if metal {
        conf::AppleGfxApi::Metal
    } else {
        conf::AppleGfxApi::OpenGl
    };

    miniquad::start(conf, move || Box::new(Stage::new()));
}

mod shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 in_pos;
    attribute vec2 in_uv;
    attribute vec2 in_instance_uv_offset;
    attribute vec2 in_instance_pos;

    uniform vec2 offset;

    varying lowp vec2 texcoord;

    void main() {
        gl_Position = vec4(in_pos + in_instance_pos, 0, 1);
        texcoord = in_uv + in_instance_uv_offset;
        texcoord = vec2(texcoord.x, texcoord.y);
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec2 texcoord;

    uniform sampler2D tex;

    void main() {
        gl_FragColor = texture2D(tex, texcoord);
    }"#;

    pub const METAL: &str = r#"
    #include <metal_stdlib>

    using namespace metal;

    struct Uniforms
    {
        float2 offset;
    };

    struct Vertex
    {
        float2 in_pos                   [[attribute(0)]];
        float2 in_uv                    [[attribute(1)]];
        float2 in_instance_uv_offset    [[attribute(2)]];
        float2 in_instance_pos          [[attribute(3)]];
    };

    struct RasterizerData
    {
        float4 position [[position]];
        float2 uv       [[user(locn0)]];
    };

    vertex RasterizerData vertexShader(
      Vertex v [[stage_in]], 
      constant Uniforms& uniforms [[buffer(0)]])
    {
        RasterizerData out;

        out.position = float4(v.in_pos.xy + uniforms.offset + v.in_instance_pos.xy, 0.0, 1.0);
        out.uv = v.in_uv + in_instance_uv_offset.xy;

        return out;
    }

    fragment float4 fragmentShader(RasterizerData in [[stage_in]], texture2d<float> tex [[texture(0)]], sampler texSmplr [[sampler(0)]])
    {
        return tex.sample(texSmplr, in.uv);
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["tex".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: vec![UniformDesc::new("offset", UniformType::Float2)],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub offset: (f32, f32),
    }
}