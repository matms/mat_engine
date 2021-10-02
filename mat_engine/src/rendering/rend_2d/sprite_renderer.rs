use wgpu::util::DeviceExt;

use super::instance::Instance;
use crate::{
    arena::{Arena, ArenaKey},
    rendering::wgpu_state::WgpuState,
    utils::unwrap_mut,
    EngineContext,
};

#[derive(Debug, Copy, Clone)]
pub struct Sprite {
    vertices: [super::Vertex2d; 4],
    indices: [u16; 6],
    texture: ArenaKey,
}

#[derive(Debug)]
struct CachedSprite {
    sprite: Sprite,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl CachedSprite {
    fn new_from_sprite(wgpu_state: &mut WgpuState, sprite: Sprite) -> Self {
        Self {
            sprite,
            vertex_buffer: wgpu_state.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("vertex buffer"),
                    contents: bytemuck::cast_slice(&sprite.vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                },
            ),
            index_buffer: wgpu_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("index buffer"),
                    contents: bytemuck::cast_slice(&sprite.indices),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
        }
    }
}

pub struct SpriteRenderer {
    sprites: Arena<CachedSprite>,
}

impl SpriteRenderer {
    pub fn new() -> Self {
        Self {
            sprites: Arena::new(),
        }
    }

    pub fn add_sprite(&mut self, ctx: &mut EngineContext, sprite: Sprite) -> ArenaKey {
        let wgpu_state = &mut unwrap_mut(&mut ctx.rendering_system).state;

        self.sprites
            .insert(CachedSprite::new_from_sprite(wgpu_state, sprite))
    }

    pub fn render_sprite(&mut self, sprite_key: ArenaKey, instance: Instance) {
        todo!()
    }

    pub fn render_sprite_batch(&mut self, sprite_key: ArenaKey, instances: Vec<Instance>) {
        todo!()
    }
}
