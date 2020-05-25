//!
//! # Engine Architecture
//!
//! This is a preliminary draft that serves just as much as documentation as it does
//! orientation (a plan) for development of the engine.
//!
//! The engine has fundamentally two different types of resources at the macro level,
//! *Systems*, and *Components*.
//!
//! Systems are stored inside the `EngineContext` object, which contains all systems, whether
//! initialized or uninitialized. An example of a system is `RenderingSystem`. Systems are
//! reasonably tightly coupled with eachother. Doing something that requires a System to be
//! initialized when it isn't is likely to panic.
//!
//! Components are managed (and owned) by the user application, with the expectation that users will
//! pick and choose them according to their needs. An example of a component is `Renderer2d`.
//!
//! Fundamentally, components depend on and use systems to provide their behaviors, whereas systems
//! shouldn't depend on components for their behaviors. Components therefore provide a higher level,
//! more specific, API, focused on a specific need. Using the `Renderer2d` example, we have
//! a component which depends on systems (see docs for specifics, but, in general, it is likely that
//! it will always depend on `WindowingSystem` and `RenderingSystem`), to provide for a specific need
//! (2D rendering).

#[macro_use]
mod macros;
mod typedefs;
mod utils;

pub mod application;
pub mod arena;
pub mod assets;
pub mod chrono;
pub mod context;
pub mod event;
pub mod imgui;
pub mod input;
pub mod rendering;
pub mod windowing;

pub use context::EngineContext;

const DEBUG_TRACE_ENGINE_START_AND_END: bool = false;
const DEBUG_TRACE_EVENT_LOOP_STEPS: bool = false;

/// Execute a given Application. Doesn't return, use the `Application::close()` method to
/// gracefully handle shutdown. See module `windowing` for more info.
///
/// Generic over Application type.
pub fn run<T: application::Application + 'static>() -> ! {
    if DEBUG_TRACE_ENGINE_START_AND_END {
        log::trace!("Starting mat_engine");
    }

    let mut ctx = context::EngineContext::uninit();

    let winit_ev_loop = windowing::make_winit_event_loop();

    ctx.chrono_init();

    ctx.input_init();

    ctx.windowing_init(
        windowing::make_default_winit_window(&winit_ev_loop),
        windowing::make_winit_event_loop_proxy(&winit_ev_loop),
    );

    ctx.rendering_init();

    let mut app = Box::new(T::new(&mut ctx));

    winit_ev_loop.run(move |event, _, control_flow| {
        // Immediately start the next loop once current is done, instead of waiting
        // for user input.
        // TODO: This may be useful for frame-rate limiting, I'm unsure. Need to examine.
        *control_flow = winit::event_loop::ControlFlow::Poll;

        // If the application is to be closed, we may skip everything below, and just quit.
        if let winit::event::Event::LoopDestroyed = event {
            app.close(&mut ctx);
            if DEBUG_TRACE_ENGINE_START_AND_END {
                log::trace!("Ending mat_engine");
            }
        // Even when we set *control_flow to Exit, winit still wants to go through the
        // outstanding events. If we wish to skip this, we can use force_quit to ignore
        // all the events until the quitting actually occurs.
        } else if ctx.windowing_system.as_mut().unwrap().force_quit {
            log::trace!("Force quitting... ignoring outsanding event");
            *control_flow = winit::event_loop::ControlFlow::Exit;
        // Normal event handling loop
        } else {
            // Handle winit events by type
            //
            // Cycle:
            //
            // START (NewEvents) -> Main events... -> UPDATE (MainEventsCleared) -> RENDER (RedrawRequested)
            // -> Post render step (RedrawEventsCleared) -> Loop again;
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
                // Currently we ignore the window id, is this a problem???
                winit::event::Event::WindowEvent { event, .. } => {
                    match event {
                        winit::event::WindowEvent::CloseRequested => {
                            // goes to winit::event::Event::LoopDestroyed, after processing
                            // queued/outstanding events
                            *control_flow = winit::event_loop::ControlFlow::Exit;
                        }
                        winit::event::WindowEvent::Resized(new_size) => {
                            // Note that if we don't call process_engine_events, that means this event will
                            // stay in the queue until that happens.
                            ctx.event_queue.push_event(event::Event::WindowResizeEvent(
                                event::events::WindowResizeEvent {
                                    new_inner_width: new_size.width,
                                    new_inner_height: new_size.height,
                                },
                            ));
                        }
                        winit::event::WindowEvent::ScaleFactorChanged {
                            new_inner_size, ..
                        } => {
                            // Note that if we don't call process_engine_events, that means this event will
                            // stay in the queue until that happens.
                            ctx.event_queue.push_event(event::Event::WindowResizeEvent(
                                event::events::WindowResizeEvent {
                                    new_inner_width: new_inner_size.width,
                                    new_inner_height: new_inner_size.height,
                                },
                            ));
                        }
                        ignored @ winit::event::WindowEvent::ThemeChanged(_)
                        | ignored @ winit::event::WindowEvent::Moved(_)
                        | ignored @ winit::event::WindowEvent::Destroyed => {
                            log::trace!("Ignored windowing event {:?}", ignored);
                        }
                        // Input events
                        input_evt => {
                            ctx.input_system
                                .as_mut()
                                .unwrap()
                                .receive_winit_windowing_event(input_evt);
                        }
                    };
                }
                winit::event::Event::DeviceEvent { event, .. } => {
                    ctx.input_system
                        .as_mut()
                        .unwrap()
                        .receive_winit_device_event(event);
                }

                // +-------+
                // | START |
                // +-------+
                winit::event::Event::NewEvents(_) => {
                    if DEBUG_TRACE_EVENT_LOOP_STEPS {
                        log::trace!("*** Start of Frame - after this, will run main events - winit NewEvents");
                    }

                    ctx.chrono_system.as_mut().unwrap().start_new_frame();

                    // If the queue isn't empty here, we made a serious mistake somewhere.
                    //
                    // Ensures events don't persist across frames.
                    assert!(ctx.event_queue.is_empty());
                    ctx.event_queue.push_event(event::Event::Start);
                    process_engine_events(&mut ctx, app.as_mut());
                }

                // +--------+
                // | UPDATE |
                // +--------+
                winit::event::Event::MainEventsCleared => {
                    if DEBUG_TRACE_EVENT_LOOP_STEPS {
                        log::trace!("*** Finished main events, now running UPDATE - winit MainEventsCleared");
                    }

                    // Since the Queue is FIFO, process outstanding events, with the last being PreUpdateEvent
                    ctx.event_queue.push_event(event::Event::PreUpdateEvent);
                    process_engine_events(&mut ctx, app.as_mut());

                    app.update(&mut ctx);

                    ctx.event_queue.push_event(event::Event::PostUpdateEvent);
                    process_engine_events(&mut ctx, app.as_mut());

                    // After `update()`, we want to run `render()`
                    {
                        ctx.windowing_system
                            .as_mut()
                            .unwrap()
                            .winit_window
                            .request_redraw();
                    }
                }
                // +--------+
                // | RENDER |
                // +--------+
                winit::event::Event::RedrawRequested(_) => {
                    if DEBUG_TRACE_EVENT_LOOP_STEPS {
                        log::trace!("*** Finished UPDATE, now running RENDER - winit RedrawRequested");
                    }

                    // Since the Queue is FIFO, process outstanding events, with the last being PreUpdateEvent
                    ctx.event_queue.push_event(event::Event::PreRenderEvent);
                    process_engine_events(&mut ctx, app.as_mut());

                    app.render(&mut ctx);

                    ctx.event_queue.push_event(event::Event::PostRenderEvent);
                    process_engine_events(&mut ctx, app.as_mut());
                }
                winit::event::Event::RedrawEventsCleared => {
                    if DEBUG_TRACE_EVENT_LOOP_STEPS {
                        log::trace!("*** Finished RENDER - winit RedrawEventsCleared");
                    }
                    // No need for any behavior here.
                }
                winit::event::Event::LoopDestroyed => {
                    unreachable!("Should be handled by if let");
                }
                // Ignore any event not specified above.
                // See winit docs for list of all possible events, and their meanings.
                // https://docs.rs/winit/0.22.1/winit/event/enum.Event.html
                _other => {
                    log::trace!("Winit misc. event: {:?}", _other);
                }
            }

            // Additional code that must be run for every event (excepting corner cases involving quitting).
            process_winit_event(&mut ctx, &event);
        }
    })
}

/// Event post-processor: Code that should be run for every* winit event should go here
///
/// *(except for `winit::event::Event::LoopDestroyed`)
fn process_winit_event(
    ctx: &mut context::EngineContext,
    event: &winit::event::Event<crate::windowing::Request>,
) {
    match &ctx.imgui_system {
        None => {}
        Some(_) => {
            imgui::process_winit_event(ctx, event);
        }
    }
}

fn process_engine_events<T: application::Application>(
    ctx: &mut context::EngineContext,
    app: &mut T,
) {
    while let Some(evt) = ctx.event_queue.retrieve_event() {
        //event::inform_receiver::<event::DebugEventReceiver>(ctx, evt);

        // For now only the rendering system is listening to events
        event::inform_receiver::<rendering::RenderingSystem>(ctx, evt);
        event::inform_receiver::<input::InputSystem>(ctx, evt);

        // After all systems are informed of an event, inform the App.
        // Note that acessing arbitrary events from the app is possibly
        // buggy behavior, so we may wish to restrict this further.
        event::inform_application(app, ctx, evt);
    }
}
