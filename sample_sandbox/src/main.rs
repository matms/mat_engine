use mat_engine::{arena::ArenaKey, input::button::ButtonId, rendering::rend_2d::Renderer2d};

use nalgebra_glm as glm;

struct MyApp {
    rend_2d: Renderer2d,
    tex_key: ArenaKey,
}

/// Note that while we may emit events for a lot of things, events are not the only way to be informed about things.
/// You should also note that some systems (will) offer polling (like the input system, once it is implemented).
impl mat_engine::event::ApplicationEventReceiver for MyApp {
    fn receives_event_type(evt_type: mat_engine::event::types::EventType) -> bool {
        match evt_type {
            _ => false,
        }
    }
    #[allow(unused_variables)]
    fn receive_event(
        &mut self,
        ctx: &mut mat_engine::EngineContext,
        evt: mat_engine::event::Event,
    ) {
        unreachable!();
    }
}

impl mat_engine::application::Application for MyApp {
    fn new(ctx: &mut mat_engine::context::EngineContext) -> Self {
        ctx.imgui_init();

        let mut rend_2d = Renderer2d::new(ctx);

        let mut tex_path = mat_engine::assets::get_folder_assets_path("sample_sandbox");
        tex_path.push("colorscales.png");

        let tex_key = rend_2d.create_new_texture_bind_group(
            ctx,
            mat_engine::assets::read_file_at_path_to_bytes(tex_path)
                .unwrap()
                .as_ref(),
            Some("Sample Texture"),
        );

        Self { rend_2d, tex_key }
    }

    fn update(&mut self, ctx: &mut mat_engine::context::EngineContext) {
        /*
        self.rend_2d.camera.mul_scale(0.9998);
        self.rend_2d
            .camera
            .translate_position(glm::vec2(0.0005, 0.0005));
        */

        let cam = &mut self.rend_2d.camera;

        if mat_engine::input::is_button_pressed(ctx, &ButtonId::Q) {
            cam.mul_scale(1.1);
        }
        if mat_engine::input::is_button_pressed(ctx, &ButtonId::E) {
            cam.mul_scale(0.9);
        }

        if mat_engine::input::is_button_down(ctx, &ButtonId::A) {
            cam.translate_position(glm::vec2(-1.0, 0.0))
        }
        if mat_engine::input::is_button_down(ctx, &ButtonId::D) {
            cam.translate_position(glm::vec2(1.0, 0.0))
        }
        if mat_engine::input::is_button_down(ctx, &ButtonId::W) {
            cam.translate_position(glm::vec2(0.0, 1.0))
        }
        if mat_engine::input::is_button_down(ctx, &ButtonId::S) {
            cam.translate_position(glm::vec2(0.0, -1.0))
        }

        self.rend_2d.update(ctx);

        mat_engine::imgui::update(ctx);
    }

    fn render(&mut self, ctx: &mut mat_engine::context::EngineContext) {
        // log::warn!("RENDER START");
        let mut frt = mat_engine::rendering::start_render(ctx).unwrap();

        self.rend_2d
            .render_sample_texture(ctx, &mut frt, self.tex_key);

        //Render imgui

        let input_sys_mouse_info = mat_engine::input::cursor::get_cursor_info(ctx);

        let (_, mouse_pos) = input_sys_mouse_info;

        let mut coordinate_info = String::from("Mouse outside screen, probably.");

        let delta = mat_engine::chrono::delta_time(ctx);

        if let Some(mouse_pos) = mouse_pos {
            let v: glm::Vec2 = glm::vec2(mouse_pos.x as f32, mouse_pos.y as f32);
            let w: glm::Vec2 = self.rend_2d.camera.pixel_screen_to_world_coords(&v);
            coordinate_info = format!("Screen coords {:?} correspond to world coords {:?}.", v, w);
        }

        mat_engine::imgui::add_render_fn(ctx, move |ui| {
            // See https://github.com/Gekkio/imgui-rs
            imgui::Window::new(imgui::im_str!("Hello world"))
                .size([300.0, 100.0], imgui::Condition::FirstUseEver)
                .build(&ui, || {
                    ui.text(imgui::im_str!("Hello world!"));
                    ui.separator();
                    let mouse_pos = ui.io().mouse_pos;
                    ui.text(format!(
                        "Mouse Position: ({:.1},{:.1})",
                        mouse_pos[0], mouse_pos[1]
                    ));
                    ui.text(format!(
                        "Input system mouse info: {:?}",
                        input_sys_mouse_info
                    ));
                    ui.text(coordinate_info.clone());
                    ui.separator();
                    ui.text(format!("Delta: {:.2}us", delta * 1_000_000.0));
                });

            //ui.show_demo_window(&mut false);
        });

        mat_engine::imgui::render(ctx, &mut frt);

        //log::warn!("COMPLETE RENDER START");

        mat_engine::rendering::complete_render(ctx, frt);

        //log::warn!("DONE RENDERING");
    }
}

fn main() {
    flexi_logger::Logger::with_str("mat_engine=trace,sample_sandbox=trace,warn")
        .format(flexi_logger::colored_opt_format)
        .start()
        .unwrap();

    log::trace!("Starting sample_sandbox");

    mat_engine::run::<MyApp>();
}
