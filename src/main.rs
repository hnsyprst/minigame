mod ecs;
mod resources;
mod linalg;
mod component;
mod system;

use std::collections::HashSet;

use miniquad::*;
use resources::ResourceManager;
use linalg::{f32, u32};
use system::{enemy_movement_system, player_movement_system, render_system, sprite_initialization_system, tile_map_initialization_system};

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
    pressed_keys: HashSet<KeyCode>,
    world: ecs::World,
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

        // Set up level
        let mut world = ecs::World::new();
        world.register_component::<component::Transform>();
        world.register_component::<component::Sprite>();
        world.register_component::<component::PlayerControl>();
        world.register_component::<component::EnemyControl>();
        world.register_component::<component::TextureAtlas>();
        world.register_component::<component::TileMap>();

        // Create texture atlas
        // TODO: Explicitly link this to `texture`
        let texture_atlas = world.create_entity();
        world.add_component(&texture_atlas, component::TextureAtlas::new(texture_atlas_size, sprite_size)).unwrap();

        let tile_map = world.create_entity();
        world.add_component(&tile_map, component::Transform { position: f32::Vec2 { x: -1.0, y: -1.0 } }).unwrap();
        world.add_component(&tile_map, component::TileMap { texture_atlas, tiles, tiles_atlas_uv_offsets: None, tiles_positions: None }).unwrap();

        // Create player
        let player = world.create_entity();
        world.add_component(&player, component::Transform { position: f32::Vec2 { x: 0.1, y: 0.2 } }).unwrap();
        world.add_component(&player, component::Sprite { texture_atlas, atlas_sprite_index: 28, atlas_uv_offset: None }).unwrap();
        world.add_component(&player, component::PlayerControl { } ).unwrap();

        // Create enemy
        let enemy = world.create_entity();
        world.add_component(&enemy, component::Transform { position: f32::Vec2 { x: 0.5, y: 0.7 } }).unwrap();
        world.add_component(&enemy, component::Sprite { texture_atlas, atlas_sprite_index: 36, atlas_uv_offset: None }).unwrap();
        world.add_component(&enemy, component::EnemyControl { } ).unwrap();

        // Initialise Entities
        // TODO: Move this into a dedicated `init()` method once level switching is implemented
        sprite_initialization_system(&world);
        tile_map_initialization_system(&world);

        Stage {
            ctx,
            pipeline,
            bindings,
            pressed_keys,
            world,
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
    }

    fn draw(&mut self) {
        render_system(
            &self.world,
            &mut self.ctx,
            &self.bindings,
            &self.pipeline,
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