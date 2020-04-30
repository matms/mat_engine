use std::cell::RefCell;

struct MyApp {
    time: std::time::SystemTime,
}

impl mat_engine::application::Application for MyApp {
    fn init(&mut self, engine: &mut mat_engine::systems::Engine) {
        self.time = std::time::SystemTime::now();

        let imgui_sys = Some(RefCell::new(mat_engine::imgui::ImguiSystem::new(engine)));

        engine.systems_borrow_mut().set_imgui(imgui_sys);
    }

    fn update(&mut self, engine: &mut mat_engine::systems::Engine) {
        engine
            .systems_borrow()
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

        engine
            .systems_borrow()
            .rendering_mut()
            .unwrap()
            .start_render();

        //Render imgui

        engine
            .systems_borrow()
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

        engine
            .systems_borrow()
            .imgui_mut()
            .expect("Imgui sys not init.")
            .render();

        engine
            .systems_borrow()
            .rendering_mut()
            .expect("Rendering sys not init.")
            .complete_render();
    }

    fn event_postprocessor(
        &mut self,
        engine: &mut mat_engine::systems::Engine,
        event: &winit::event::Event<mat_engine::windowing::Request>,
    ) {
        engine
            .systems_borrow()
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
