use mat_engine::{arena::ArenaKey, rendering::rend_2d::Renderer2d};

use nalgebra_glm as glm;

struct MyApp {
    time: std::time::SystemTime,
    rend_2d: Option<Renderer2d>,
    tex_key: ArenaKey,
}

impl mat_engine::application::Application for MyApp {
    fn init(&mut self, ctx: &mut mat_engine::context::EngineContext) {
        //log::warn!("APP INIT!");

        self.time = std::time::SystemTime::now();

        ctx.imgui_init();

        self.rend_2d = Some(Renderer2d::new(ctx));

        self.tex_key = self
            .rend_2d
            .as_mut()
            .unwrap()
            .create_new_texture_bind_group(
                ctx,
                include_bytes!("colorscales.png"),
                Some("Sample Texture"),
            );
    }

    fn update(&mut self, ctx: &mut mat_engine::context::EngineContext) {
        self.rend_2d.as_mut().unwrap().camera.mul_scale(0.9998);
        self.rend_2d
            .as_mut()
            .unwrap()
            .camera
            .translate_position(glm::vec2(0.0005, 0.0005));

        self.rend_2d.as_mut().unwrap().update(ctx);

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
            .as_mut()
            .unwrap()
            .render_sample_texture(ctx, &mut frt, self.tex_key);

        //Render imgui

        // Copy dur (Duration is a Copy type).
        let _dur = dur;

        mat_engine::imgui::add_render_fn(ctx, move |ui| {
            // See https://github.com/Gekkio/imgui-rs
            imgui::Window::new(imgui::im_str!("Hello world"))
                .size([300.0, 100.0], imgui::Condition::FirstUseEver)
                .build(&ui, || {
                    ui.text(imgui::im_str!("Hello world!"));
                    ui.text(imgui::im_str!("This...is...imgui-rs!"));
                    ui.separator();
                    let mouse_pos = ui.io().mouse_pos;
                    ui.text(format!(
                        "Mouse Position: ({:.1},{:.1})",
                        mouse_pos[0], mouse_pos[1]
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

    mat_engine::run(Box::new(MyApp {
        time: std::time::SystemTime::UNIX_EPOCH,
        rend_2d: None,
        tex_key: ArenaKey::default(),
    }));
}
