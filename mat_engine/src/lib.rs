#[macro_use]
mod macros;
mod typedefs;
mod utils;

pub mod application;
pub mod arena;
pub mod context;
pub mod imgui;
pub mod rendering;
pub mod windowing;

pub use context::EngineContext;

/// Execute a given Application. Doesn't return, use the `Application::close()` method to
/// handle shutdown.
pub fn run(mut app: Box<dyn application::Application>) -> ! {
    log::trace!("Starting mat_engine");

    let mut ctx = context::EngineContext::uninit();

    let ev_loop = windowing::make_winit_event_loop();

    ctx.windowing_init(
        windowing::make_default_winit_window(&ev_loop),
        windowing::make_winit_event_loop_proxy(&ev_loop),
    );

    ctx.rendering_init();

    app.init(&mut ctx);

    ev_loop.run(move |event, _, control_flow| {
        // Immediately start the next loop once current is done, instead of waiting
        // for user input.
        // TODO: This may be useful for frame-rate limiting, I'm unsure. Need to examine.
        *control_flow = winit::event_loop::ControlFlow::Poll;

        // If the application is to be closed, we may skip everything below, and just quit.
        if let winit::event::Event::LoopDestroyed = event {
            app.close(&mut ctx);
            log::trace!("Ending mat_engine");
        // Even when we set *control_flow to Exit, winit still wants to go through the
        // outstanding events. If we wish to skip this, we can use force_quit to ignore
        // all the events until the quitting actually occurs.
        } else if ctx.windowing_system.as_mut().unwrap().force_quit {
            log::trace!("Force quitting... ignoring outsanding event");
            *control_flow = winit::event_loop::ControlFlow::Exit;
        } else {
            match &event {
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
                    windowing::notify_resize(&mut ctx, new_size.width, new_size.height);
                }
                winit::event::Event::WindowEvent {
                    event: winit::event::WindowEvent::ScaleFactorChanged { new_inner_size, .. },
                    ..
                } => {
                    windowing::notify_resize(&mut ctx, new_inner_size.width, new_inner_size.height);
                }
                // --------------------------------------------------
                winit::event::Event::MainEventsCleared => {
                    app.update(&mut ctx);

                    {
                        ctx.windowing_system
                            .as_mut()
                            .unwrap()
                            .winit_window
                            .request_redraw();
                    }
                }
                winit::event::Event::RedrawRequested(_) => {
                    app.render(&mut ctx);
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

            process_event(&mut ctx, &event);
        }
    })
}

/// Event post-processor: Code that should be run for every* event should go here
///
/// *(except for `winit::event::Event::LoopDestroyed`)
fn process_event(
    ctx: &mut context::EngineContext,
    event: &winit::event::Event<crate::windowing::Request>,
) {
    match &ctx.imgui_system {
        None => {}
        Some(_) => {
            imgui::process_event(ctx, event);
        }
    }
}
