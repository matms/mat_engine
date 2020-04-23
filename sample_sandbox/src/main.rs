struct MyApp {
    initialized_systems: Option<mat_engine::application::InitializedSystems>,
    time: std::time::SystemTime,
    imgui_system: Option<mat_engine::imgui::ImguiSystem>,
}

impl mat_engine::application::Application for MyApp {
    fn init(&mut self, initialized_systems: mat_engine::application::InitializedSystems) {
        self.initialized_systems = Some(initialized_systems);
        self.time = std::time::SystemTime::now();
        self.imgui_system = Some(mat_engine::imgui::ImguiSystem::new(
            &mut self
                .initialized_systems
                .as_ref()
                .unwrap()
                .windowing_system
                .borrow_mut(),
            &mut self
                .initialized_systems
                .as_ref()
                .unwrap()
                .rendering_system
                .borrow_mut(),
        ))
    }

    fn update(&mut self) {
        self.imgui_system.as_mut().unwrap().update(
            &mut self
                .initialized_systems
                .as_ref()
                .unwrap()
                .windowing_system
                .borrow_mut(),
        );
    }

    fn render(&mut self) {
        let new_time = std::time::SystemTime::now();
        let _dur = new_time
            .duration_since(self.time)
            .expect("System time progressed non-monotonically, I think");
        self.time = new_time;

        //log::trace!("Last frame duration {}us", _dur.as_micros());

        self.initialized_systems
            .as_ref()
            .unwrap()
            .rendering_system
            .borrow_mut()
            .start_render();

        //Render imgui

        self.imgui_system.as_mut().unwrap().add_render_fn(|ui| {
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

        self.imgui_system.as_mut().unwrap().render(
            &mut self
                .initialized_systems
                .as_ref()
                .unwrap()
                .windowing_system
                .borrow_mut(),
            &mut self
                .initialized_systems
                .as_ref()
                .unwrap()
                .rendering_system
                .borrow_mut(),
        );

        self.initialized_systems
            .as_ref()
            .unwrap()
            .rendering_system
            .borrow_mut()
            .complete_render();
    }

    fn event_postprocessor(&mut self, event: &winit::event::Event<mat_engine::windowing::Request>) {
        self.imgui_system.as_mut().unwrap().process_event(
            event,
            &mut self
                .initialized_systems
                .as_ref()
                .unwrap()
                .windowing_system
                .borrow_mut(),
        );
    }
}

fn main() {
    flexi_logger::Logger::with_str("mat_engine=trace,sample_sandbox=trace,warn")
        .format(flexi_logger::colored_opt_format)
        .start()
        .unwrap();

    log::trace!("Starting sample_sandbox");
    mat_engine::run(Box::new(MyApp {
        initialized_systems: None,
        time: std::time::SystemTime::UNIX_EPOCH,
        imgui_system: None,
    }));
}
