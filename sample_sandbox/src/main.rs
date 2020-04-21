struct MyApp {
    initialized_systems: Option<mat_engine::application::InitializedSystems>,
}

impl mat_engine::application::Application for MyApp {
    fn init(&mut self, initialized_systems: mat_engine::application::InitializedSystems) {
        self.initialized_systems = Some(initialized_systems);
    }

    fn update(&mut self) {
        // TODO: Is this the best way to do this??? I suspect not... Investigate.
        self.initialized_systems
            .as_ref()
            .expect("initialized_systems not initialized")
            .windowing_system
            .borrow_mut()
            .force_quit();

        log::trace!("requesting quit");
    }

    fn render(&mut self) {
        log::trace!("rend");
    }
}

fn main() {
    flexi_logger::Logger::with_str("trace")
        .format(flexi_logger::colored_opt_format)
        .start()
        .unwrap();

    log::trace!("Starting sample_sandbox");
    mat_engine::run(Box::new(MyApp {
        initialized_systems: None,
    }));
}
