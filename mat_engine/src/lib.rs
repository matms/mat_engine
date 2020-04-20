#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

//TODO: Refactor

///Temporary main function
pub fn temporary_main() {
    log::trace!("Starting mat_engine");

    let winit_event_loop = winit::event_loop::EventLoop::new();
    let winit_window = winit::window::WindowBuilder::new()
        .build(&winit_event_loop)
        .expect("Could not obtain winit window");

    winit_event_loop.run(move |event, _, control_flow| {
        // Immediately start the next loop once current is done, instead of waiting
        // for user input.
        // TODO: This may be useful for frame-rate limiting, I'm unsure. Need to examine.
        *control_flow = winit::event_loop::ControlFlow::Poll;

        match event {
            winit::event::Event::NewEvents(start_cause) => {
                //log::trace!("winit loop -> NewEvents(start_cause = {:?})", start_cause);
            }
            winit::event::Event::WindowEvent { window_id, event } => {
                match event {
                    winit::event::WindowEvent::CloseRequested => {
                        //log::trace!("winit window CloseRequested event, closing.");
                        *control_flow = winit::event_loop::ControlFlow::Exit;
                    }
                    other_event => {
                        log::trace!("winit window event, specifically {:?}", other_event);
                    }
                };
            }
            winit::event::Event::DeviceEvent { device_id, event } => {}
            winit::event::Event::UserEvent(_) => {
                log::error!("winit UserEvent is not (currently?) supported.");
                unimplemented!();
            }
            winit::event::Event::Suspended => {
                log::trace!("winit loop -> application suspended");
            }
            winit::event::Event::Resumed => {
                log::trace!("winit loop -> application resumed");
            }
            winit::event::Event::MainEventsCleared => {
                //log::trace!("TODO update goes here...");

                // Once we finish updating, we want to redraw.
                // TODO: Do we??? Also: Framerate limiting.
                winit_window.request_redraw();
            }
            winit::event::Event::RedrawRequested(_) => {
                //log::trace!("TODO render goes here");
            }
            winit::event::Event::RedrawEventsCleared => {
                //log::trace!("TODO post render goes here");
            }
            winit::event::Event::LoopDestroyed => {
                //log::trace!("winit event loop quitting");
            }
        }
    })
}
