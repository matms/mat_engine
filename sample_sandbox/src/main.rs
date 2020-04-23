use std::cell::RefCell;

struct MyApp {
    time: std::time::SystemTime,
}

impl mat_engine::application::Application for MyApp {
    fn init(&mut self, engine: &mut mat_engine::systems::Engine) {
        self.time = std::time::SystemTime::now();

        let sys_rc = engine
            .systems_ref()
            .upgrade()
            .expect("Failed to get systems, maybe the Engine has been dropped");

        let imgui_sys = Some(RefCell::new(mat_engine::imgui::ImguiSystem::new(
            engine.systems_ref(),
        )));

        let mut systems_mut = sys_rc.borrow_mut();
        systems_mut.set_imgui(imgui_sys);
    }

    fn update(&mut self, engine: &mut mat_engine::systems::Engine) {
        let sys_rc = engine
            .systems_ref()
            .upgrade()
            .expect("Failed to get systems, maybe the Engine has been dropped");
        let systems_ref = sys_rc.borrow();

        systems_ref
            .imgui_mut()
            .expect("Imgui system not init.")
            .update();
    }

    fn render(&mut self, engine: &mut mat_engine::systems::Engine) {
        let new_time = std::time::SystemTime::now();
        let _dur = new_time
            .duration_since(self.time)
            .expect("System time progressed non-monotonically, I think");
        self.time = new_time;

        //log::trace!("Last frame duration {}us", _dur.as_micros());

        let sys_rc = engine
            .systems_ref()
            .upgrade()
            .expect("Failed to get systems, maybe the Engine has been dropped");
        let systems_ref = sys_rc.borrow();

        systems_ref
            .rendering_mut()
            .expect("Render sys not init.")
            .start_render();

        //Render imgui

        systems_ref
            .imgui_mut()
            .expect("Imgui sys not init.")
            .add_render_fn(|ui| {
                // See https://github.com/Gekkio/imgui-rs
                imgui::Window::new(imgui::im_str!("Hello world"))
                    .size([300.0, 100.0], imgui::Condition::FirstUseEver)
                    .build(&ui, || {
                        ui.text(imgui::im_str!("Hello world!"));
                        ui.text(imgui::im_str!("こんにちは世界！"));
                        ui.text(imgui::im_str!("This...is...imgui-rs!"));
                        ui.separator();
                        let mouse_pos = ui.io().mouse_pos;
                        ui.text(format!(
                            "Mouse Position: ({:.1},{:.1})",
                            mouse_pos[0], mouse_pos[1]
                        ));
                    });
            });

        systems_ref
            .imgui_mut()
            .expect("Imgui sys not init.")
            .render();

        systems_ref
            .rendering_mut()
            .expect("Rendering sys not init.")
            .complete_render();
    }

    fn event_postprocessor(
        &mut self,
        engine: &mut mat_engine::systems::Engine,
        event: &winit::event::Event<mat_engine::windowing::Request>,
    ) {
        let sys_rc = engine
            .systems_ref()
            .upgrade()
            .expect("Failed to get systems, maybe the Engine has been dropped");
        let systems_ref = sys_rc.borrow();

        systems_ref
            .imgui_mut()
            .expect("Imgui sys not init.")
            .process_event(event);
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
    }));
}
