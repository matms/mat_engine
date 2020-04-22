use std::{cell::RefCell, rc::Rc};

pub mod application;
pub mod render;
pub mod windowing;

/// Execute a given Application. Doesn't return, use the `Application::close()` method to
/// handle shutdown.
pub fn run(mut app: Box<dyn application::Application>) -> ! {
    log::trace!("Starting mat_engine");

    let winit_event_loop = winit::event_loop::EventLoop::<windowing::Request>::with_user_event();
    let winit_event_loop_proxy = winit_event_loop.create_proxy();
    let winit_window = winit::window::WindowBuilder::new()
        .build(&winit_event_loop)
        .expect("Could not obtain winit window");

    let windowing_system = Rc::new(RefCell::new(windowing::WindowingSystem {
        winit_window,
        winit_event_loop_proxy,
        force_quit: false,
        resize_listeners: vec![],
    }));

    let rendering_system =
        crate::render::RenderingSystem::new(&mut windowing_system.as_ref().borrow_mut());

    let initialized_systems = application::InitializedSystems {
        windowing_system: windowing_system.clone(),
        rendering_system: rendering_system.clone(),
    };

    app.init(initialized_systems);

    winit_event_loop.run(move |event, _, control_flow| {
        // Immediately start the next loop once current is done, instead of waiting
        // for user input.
        // TODO: This may be useful for frame-rate limiting, I'm unsure. Need to examine.
        *control_flow = winit::event_loop::ControlFlow::Poll;

        // If the application is to be closed, we may skip everything below, and just quit.
        if let winit::event::Event::LoopDestroyed = event {
            app.close();
            log::trace!("Ending mat_engine");
        // Even when we set *control_flow to Exit, winit still wants to go through the
        // outstanding events. If we wish to skip this, we can use force_quit to ignore
        // all the events until the quitting actually occurs.
        } else if windowing_system.borrow_mut().force_quit {
            log::trace!("Force quitting... ignoring outsanding event");
            *control_flow = winit::event_loop::ControlFlow::Exit;
        } else {
            match event {
                winit::event::Event::UserEvent(request) => {
                    match request {
                        windowing::Request::Quit => {
                            // goes to winit::event::Event::LoopDestroyed, after processing
                            // queued/outstanding events
                            *control_flow = winit::event_loop::ControlFlow::Exit;
                        }
                    };
                }
                winit::event::Event::WindowEvent {
                    event: winit::event::WindowEvent::CloseRequested,
                    ..
                } => {
                    // goes to winit::event::Event::LoopDestroyed, after processing
                    // queued/outstanding events
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
                // --------------------------------------------------
                // +----------+
                // | RESIZING |
                // +----------+
                winit::event::Event::WindowEvent {
                    event: winit::event::WindowEvent::Resized(new_size),
                    ..
                } => {
                    windowing_system
                        .borrow()
                        .notify_resize(new_size.width, new_size.height);
                }
                winit::event::Event::WindowEvent {
                    event: winit::event::WindowEvent::ScaleFactorChanged { new_inner_size, .. },
                    ..
                } => {
                    windowing_system
                        .borrow()
                        .notify_resize(new_inner_size.width, new_inner_size.height);
                }
                // --------------------------------------------------
                winit::event::Event::MainEventsCleared => {
                    app.update();
                    {
                        windowing_system.borrow_mut().winit_window.request_redraw();
                    }
                }
                winit::event::Event::RedrawRequested(_) => {
                    app.render();
                }
                winit::event::Event::LoopDestroyed => {
                    unreachable!("Should be handled by if let");
                }
                // Ignore any event not specified above.
                // See winit docs for list of all possible events, and their meanings.
                // https://docs.rs/winit/0.22.1/winit/event/enum.Event.html
                _other => {
                    //log::trace!("Winit misc. event: {:?}", other);
                }
            }
        }
    })
}
