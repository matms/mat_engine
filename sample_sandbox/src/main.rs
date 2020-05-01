struct MyApp {
    time: std::time::SystemTime,
}

impl mat_engine::application::Application for MyApp {
    fn init(&mut self, ctx: &mut mat_engine::context::EngineContext) {
        self.time = std::time::SystemTime::now();
        ctx.imgui_init();
    }

    fn update(&mut self, ctx: &mut mat_engine::context::EngineContext) {
        mat_engine::imgui::update(ctx);
    }

    fn render(&mut self, ctx: &mut mat_engine::context::EngineContext) {
        let new_time = std::time::SystemTime::now();
        let _dur = new_time
            .duration_since(self.time)
            .expect("System time progressed non-monotonically, I think");
        self.time = new_time;

        //log::trace!("Last frame duration {}us", _dur.as_micros());

        mat_engine::rendering::start_render(ctx);

        //Render imgui
        mat_engine::imgui::add_render_fn(ctx, |ui| {
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

        mat_engine::imgui::render(ctx);

        mat_engine::rendering::complete_render(ctx);
    }

    fn event_postprocessor(
        &mut self,
        ctx: &mut mat_engine::context::EngineContext,
        event: &winit::event::Event<mat_engine::windowing::Request>,
    ) {
        mat_engine::imgui::process_event(ctx, event);
    }
}

fn main() {
    flexi_logger::Logger::with_str("mat_engine=trace,sample_sandbox=trace,warn")
        .format(flexi_logger::colored_opt_format)
        .start()
        .unwrap();

    log::trace!("Starting random tests");
    random_testing();
    log::trace!("Ending random tests");

    log::trace!("Starting sample_sandbox");

    mat_engine::run(Box::new(MyApp {
        time: std::time::SystemTime::UNIX_EPOCH,
    }));
}

trait Testing {
    fn quack(&self) -> String;
}

struct A;

impl Testing for A {
    fn quack(&self) -> String {
        "Quack".into()
    }
}
struct B;

impl Testing for B {
    fn quack(&self) -> String {
        "I'm not a duck".into()
    }
}

fn random_testing() {
    let mut a = mat_engine::slotmap::Arena::<Box<dyn Testing>>::new();
    let foo = a.insert(Box::new(A {}));
    let bar = a.insert(Box::new(B {}));
    *a.get_mut(foo).unwrap() = Box::new(B {});
    *a.get_mut(bar).unwrap() = Box::new(A {});
    log::info!(
        "a[foo] = {}, a[bar]={}",
        a.get(foo).unwrap().quack(),
        a.get(bar).unwrap().quack()
    )
}
