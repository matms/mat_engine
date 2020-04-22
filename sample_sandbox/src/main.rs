struct MyApp {
    initialized_systems: Option<mat_engine::application::InitializedSystems>,
    time: std::time::SystemTime,
}

impl mat_engine::application::Application for MyApp {
    fn init(&mut self, initialized_systems: mat_engine::application::InitializedSystems) {
        self.initialized_systems = Some(initialized_systems);
        self.time = std::time::SystemTime::now();
    }

    fn render(&mut self) {
        let new_time = std::time::SystemTime::now();
        let dur = new_time
            .duration_since(self.time)
            .expect("System time progressed non-monotonically, I think");
        self.time = new_time;

        log::trace!("Last frame duration {}us", dur.as_micros());

        self.initialized_systems
            .as_ref()
            .unwrap()
            .rendering_system
            .borrow_mut()
            .render();
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
    }));
}
