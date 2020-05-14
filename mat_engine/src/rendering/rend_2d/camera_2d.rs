use crate::{
    arena::ArenaKey,
    rendering::{bind_group::BindGroupable, generic_uniform::Uniform, wgpu_state::WgpuState},
};

use nalgebra_glm as glm;

pub struct Camera2d {
    camera_uniform_component: CameraUniformComponent,
    pub(super) camera_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) camera_bind_group_key: ArenaKey,
    screen_width: u32,
    screen_height: u32,
    scale: f32,
    position: glm::Vec2,
    needs_matrix_update: bool,
    camera_matrix: glm::Mat4,
    ortho_matrix: glm::Mat4,
}

/// Default camera for `Renderer2d`. Permits translation and scaling of the camera, but not rotation.
impl Camera2d {
    pub(super) fn new(screen_width: u32, screen_height: u32, wgpu_state: &mut WgpuState) -> Self {
        let camera_uniform_component = CameraUniformComponent::new(&mut wgpu_state.device);

        let camera_desc = CameraUniformComponent::get_wgpu_bind_group_layout_descriptor();

        let camera_bind_group_layout = wgpu_state.device.create_bind_group_layout(&camera_desc);

        let camera_bind_group_key = wgpu_state.add_new_uniform_bind_group(
            &camera_bind_group_layout,
            &camera_uniform_component,
            Some("camera_uniform_bind_group"),
        );

        let mut out = Self {
            camera_uniform_component,
            camera_bind_group_layout,
            camera_bind_group_key,
            screen_width,
            screen_height,
            scale: 1.0, // TODO: Adjust
            position: glm::vec2(0.0, 0.0),
            needs_matrix_update: true,
            camera_matrix: glm::identity(),
            ortho_matrix: glm::identity(),
        };

        out.update(wgpu_state);

        out
    }

    /// Temporary. TODO: Replace once event system has been implemented.
    ///
    /// Updates screen size if it is different from what camera had stored.
    pub(super) fn feed_screen_size(&mut self, screen_width: u32, screen_height: u32) {
        if (self.screen_width != screen_width) || (self.screen_height != screen_height) {
            self.set_screen_size(screen_width, screen_height);
        }
    }

    /// Sets screen size. You should probably use `feed_screen_size()` instead.
    ///
    /// We use setters because we need to flag that the camera matrix should be
    /// updated. The actual updating of the camera matrix happens on `Camera2d::update()`.
    pub(super) fn set_screen_size(&mut self, screen_width: u32, screen_height: u32) {
        self.screen_width = screen_width;
        self.screen_height = screen_height;
        self.needs_matrix_update = true;
    }

    /// Sets the camera scale factor. Bigger is more zoomed in.
    ///
    /// We use setters because we need to flag that the camera matrix should be
    /// updated. The actual updating of the camera matrix happens on `Camera2d::update()`.
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
        self.needs_matrix_update = true;
    }

    /// Multiplies (scales) the camera scale factor. Needed because we don't (as of the time this
    /// has been written) expose getters.
    ///
    /// We use setters because we need to flag that the camera matrix should be
    /// updated. The actual updating of the camera matrix happens on `Camera2d::update()`.
    pub fn mul_scale(&mut self, factor: f32) {
        self.set_scale(self.scale * factor);
    }

    /// Sets the camera's position in the world.
    ///
    /// We use setters because we need to flag that the camera matrix should be
    /// updated. The actual updating of the camera matrix happens on `Camera2d::update()`.
    pub fn set_position(&mut self, position: glm::Vec2) {
        self.position = position;
        self.needs_matrix_update = true;
    }

    /// Adds to (translates) the camera's position in the world. Needed because we don't
    /// (as of the time this has been written) expose getters.
    ///
    /// We use setters because we need to flag that the camera matrix should be
    /// updated. The actual updating of the camera matrix happens on `Camera2d::update()`.
    pub fn translate_position(&mut self, add: glm::Vec2) {
        self.set_position(self.position + add);
    }

    /// Given a world coordinate, computes the corresponding screen coordinate (measured in pixels,
    /// with (0,0) at the top left). The world coordinate is the coordinate that is multiplied
    /// by the camera matrix to calculate the coordinates used for rendering.
    ///
    /// Assumes the camera has been fed the correct screen size.
    ///
    /// Note that the screen coordinate **in pixels** doesn't correspond to the `gl_position` attribute
    /// as computed in shaders (which has a specific range). This is therefore NOT a replacement
    /// for using the projection matrix. Sending pixel coordinates to the renderer directly will likely
    /// not work.
    ///
    /// Does not access the matrix. Instead, it uses the camera's position, scale, and information about
    /// the screen's size directly.
    pub fn world_to_pixel_screen_coords(&self, world_coords: &glm::Vec2) -> glm::Vec2 {
        let sh = self.screen_height as f32;
        let sw = self.screen_width as f32;

        // Handle camera translation and scaling
        let mut out: glm::Vec2 = world_coords - self.position;
        out *= self.scale;

        // Center is at middle of screen, by default
        out += glm::vec2(sw / 2.0, sh / 2.0);

        // Invert y axis: (The screen has y positive downwards while the world has y positive upwards)
        out.y = sh - out.y;

        out
    }

    /// Given a screen coordinate (measured in pixels, with (0,0) at the top left), computes
    /// the corresponding world coordinate. The world coordinate is the coordinate that is multiplied
    /// by the camera matrix to calculate the coordinates used for rendering.
    ///
    /// Assumes the camera has been fed the correct screen size, and that the current screen size
    /// corresponds to the screen size at the time of measurement.
    ///
    /// Note that the screen coordinate **in pixels** doesn't correspond to the `gl_position` attribute
    /// as computed in shaders.
    ///
    /// Does not access the matrix. Instead, it uses the camera's position, scale, and information about
    /// the screen's size directly.
    pub fn pixel_screen_to_world_coords(&self, pixel_screen_coords: &glm::Vec2) -> glm::Vec2 {
        let sh = self.screen_height as f32;
        let sw = self.screen_width as f32;

        // Invert y axis: (The screen has y positive downwards while the world has y positive upwards)
        let mut out: glm::Vec2 = glm::vec2(pixel_screen_coords.x, sh - pixel_screen_coords.y);

        // Center is at middle of screen, by default
        out -= glm::vec2(sw / 2.0, sh / 2.0);

        // Handle camera translation and scaling
        out /= self.scale;
        out += self.position;

        out
    }

    /// Updates the camera. Should be called on the application's loop `update()` function.
    ///
    /// Recomputes the matrix if any relevant parameters have changed, and, if so, also sends the
    /// new matrix over to the GPU. The latter step is why this takes in `wgpu_state`.
    ///
    /// TODO: wrap this in a function that takes in EngineContext.
    pub(super) fn update(&mut self, wgpu_state: &mut WgpuState) {
        if self.needs_matrix_update {
            self.update_ortho_matrix();
            self.update_camera_matrix();
            self.update_uniform(wgpu_state);

            self.needs_matrix_update = false;
        }
    }

    fn update_ortho_matrix(&mut self) {
        self.ortho_matrix = glm::ortho(
            0.0,
            self.screen_width as f32,
            0.0,
            self.screen_height as f32,
            -1.0,
            1.0,
        );
    }

    fn update_camera_matrix(&mut self) {
        let translate: glm::Vec3 = glm::vec3(
            -self.position.x + (self.screen_width as f32) / 2.0,
            -self.position.y + (self.screen_height as f32) / 2.0,
            0.0,
        );

        let scale: glm::Vec3 = glm::vec3(self.scale, self.scale, 0.0);

        self.camera_matrix = glm::translate(&self.ortho_matrix, &translate);
        self.camera_matrix = glm::scale(&glm::identity(), &scale) * self.camera_matrix;
    }

    fn update_uniform(&mut self, wgpu_state: &mut WgpuState) {
        self.camera_uniform_component.content.projection_matrix = self.camera_matrix;

        wgpu_state.update_uniform_buffer(&self.camera_uniform_component);
    }
}

pub(super) struct CameraUniformComponent {
    content: CameraUniformContent,
    buffer: wgpu::Buffer,
}

impl CameraUniformComponent {
    pub(super) fn new(device: &mut wgpu::Device) -> Self {
        let content = CameraUniformContent {
            projection_matrix: glm::identity(),
        };
        let buffer = Self::create_new_buffer(
            content,
            device,
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );
        Self { content, buffer }
    }
}

#[derive(Copy, Clone, Debug)]
pub(super) struct CameraUniformContent {
    projection_matrix: glm::Mat4,
}

// Asserts that there is no padding in CameraUniformContent
static_assertions::const_assert_eq!(
    std::mem::size_of::<CameraUniformContent>(),
    std::mem::size_of::<f32>() * 4 * 4
);

// Safety:
// See https://docs.rs/bytemuck/1.2.0/bytemuck/trait.Zeroable.html
// We know that Mat4 is just 16 f32 values, so it can be zeroed.
unsafe impl bytemuck::Zeroable for CameraUniformContent {}

// Safety:
// See https://docs.rs/bytemuck/1.2.0/bytemuck/trait.Pod.html
// We need to check for the absence of padding, see static assert above.
unsafe impl bytemuck::Pod for CameraUniformContent {}

// TODO: A lot from  here on down was copy pasted. See if I can abstract it away...

impl BindGroupable for CameraUniformComponent {
    fn get_wgpu_bind_group_layout_descriptor() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            bindings: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false, // This is NOT a dynamically sized array, it is statically sized.
                },
            }],
            label: Some("camera_bind_group_layout"),
        }
    }
    fn make_wgpu_bind_group(
        &self,
        bind_group_layout: &wgpu::BindGroupLayout,
        device: &mut wgpu::Device,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &self.buffer,
                    range: 0..std::mem::size_of_val(&self.content) as wgpu::BufferAddress,
                },
            }],
            label: Some("camera_bind_group"),
        })
    }
}

impl Uniform for CameraUniformComponent {
    type Content = CameraUniformContent;

    fn create_new_buffer(
        content: Self::Content,
        device: &mut wgpu::Device,
        usage: wgpu::BufferUsage,
    ) -> wgpu::Buffer {
        device.create_buffer_with_data(bytemuck::cast_slice(&[content]), usage)
    }
    fn update_buffer(&self, encoder: &mut wgpu::CommandEncoder, device: &mut wgpu::Device) {
        let staging_buffer =
            Self::create_new_buffer(self.content, device, wgpu::BufferUsage::COPY_SRC);

        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.buffer,
            0,
            std::mem::size_of_val(&self.content) as wgpu::BufferAddress,
        );
    }
}
