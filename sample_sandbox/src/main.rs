use mat_engine::{arena::ArenaKey, input::button::ButtonId, rendering::rend_2d::Renderer2d};

use nalgebra_glm as glm;

struct MyApp {
    time: std::time::SystemTime,
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
        let time = std::time::SystemTime::now();

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

        Self {
            time,
            rend_2d,
            tex_key,
        }
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

        // We mustn't forget this...
        mat_engine::input::finished_reading_input(ctx);

        self.rend_2d.update(ctx);

        mat_engine::imgui::update(ctx);
    }

    fn render(&mut self, ctx: &mut mat_engine::context::EngineContext) {
        let new_time = std::time::SystemTime::now();
        let dur = new_time
            .duration_since(self.time)
            .expect("System time progressed non-monotonically, I think");

        self.time = new_time;

        //log::warn!("RENDER START");
        let mut frt = mat_engine::rendering::start_render(ctx);

        self.rend_2d
            .render_sample_texture(ctx, &mut frt, self.tex_key);

        let a = glm::vec2(0.0, 30.0);
        /*log::trace!(
            "A) World coords {:?} correspond to pixel screen coords {:?}",
            a,
            self.rend_2d.camera.world_to_pixel_screen_coords(&a)
        );*/

        let b = glm::vec2(512.0, 355.0);
        /*log::trace!(
            "C) Pixel screen coords {:?} correspond to world coords {:?}",
            b,
            self.rend_2d.camera.pixel_screen_to_world_coords(&b)
        );*/

        //Render imgui

        // Copy dur (Duration is a Copy type).
        let _dur = dur;

        let input_sys_mouse_info = mat_engine::input::cursor::get_cursor_info(ctx);

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
                    ui.separator();
                    ui.text(format!("Last frame duration: {}us.", _dur.as_micros()));
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
