use battle_arena_2000::*;
use glutin::{event::Event, event::WindowEvent, event_loop::ControlFlow, Api, GlRequest};

fn main() -> Result<(), ()> {
    let event_loop = glutin::event_loop::EventLoop::new();
    let window_builder = glutin::window::WindowBuilder::new()
        .with_title("Hello world!")
        .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));

    let windowed_context = glutin::ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .build_windowed(window_builder, &event_loop)
        .expect("Context creation failed");

    let gl_current = unsafe { windowed_context.make_current().expect("Make current fail") };

    unsafe {
        gl::load_with(|symbol| gl_current.get_proc_address(symbol) as *const _);
        gl::Viewport(0, 0, 1024, 768)
    };

    let mut dpi = gl_current.window().scale_factor();

    let mut game = Game::new();
    game.add_player();

    //++++++++++++++++++++//
    //  collision entity //
    //++++++++++++++++++//
    // let collision_entity = entity_manager.next_entity();
    // component_manager.set_position_component(
    //     collision_entity,
    //     PositionComponent::new_wrapping(250.0f32, 250.0),
    // );
    // let collision_size = 60.0;
    // component_manager.set_render_component(collision_entity, unsafe {
    //     RenderComponent::new_circle(collision_size)
    // });
    // component_manager
    //     .set_collision_component(collision_entity, CollisionComponent::new(collision_size));
    // component_manager.set_body_component(collision_entity, BodyComponent::new(20.0, 0.4));

    let mut last_instant = std::time::Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            // Application update code.

            let new_instant = std::time::Instant::now();
            let dt = new_instant - last_instant;
            last_instant = new_instant;

            game.update_state(dt);

            // Queue a RedrawRequested event.
            gl_current.window().request_redraw();
        }
        Event::RedrawRequested { .. } => {
            // Redraw the application.
            //
            // It's preferrable to render in this event rather than in EventsCleared, since
            // rendering in here allows the program to gracefully handle redraws requested
            // by the OS.
            //
            // render_system.render(&arena, &component_manager);
            game.render();

            gl_current.swap_buffers().unwrap();
        }
        Event::WindowEvent {
            event: WindowEvent::ScaleFactorChanged { scale_factor, .. },
            ..
        } => dpi = scale_factor,
        Event::WindowEvent {
            event: WindowEvent::Resized(size),
            ..
        } => {
            // FIXME: this is probably not ok (?)
            gl_current.resize(size.to_logical::<u32>(dpi).to_physical(dpi));
        }
        Event::WindowEvent {
            event: WindowEvent::KeyboardInput { input, .. },
            ..
        } => {
            if let Some(key_code) = input.virtual_keycode {
                use glutin::event::ElementState;
                match (key_code, input.state) {
                    (glutin::event::VirtualKeyCode::Escape, ElementState::Pressed) => {
                        println!("The escape key was pressed; stopping");
                        *control_flow = ControlFlow::Exit;
                    }
                    (glutin::event::VirtualKeyCode::W, ElementState::Pressed) => {
                        game.player_command(PlayerCommand::Movement {
                            direction: MovementDirection::Up,
                            action: MovementAction::Start,
                        });
                    }
                    (glutin::event::VirtualKeyCode::W, ElementState::Released) => {
                        game.player_command(PlayerCommand::Movement {
                            direction: MovementDirection::Up,
                            action: MovementAction::Stop,
                        });
                    }
                    (glutin::event::VirtualKeyCode::S, ElementState::Pressed) => {
                        game.player_command(PlayerCommand::Movement {
                            direction: MovementDirection::Down,
                            action: MovementAction::Start,
                        });
                    }
                    (glutin::event::VirtualKeyCode::S, ElementState::Released) => {
                        game.player_command(PlayerCommand::Movement {
                            direction: MovementDirection::Down,
                            action: MovementAction::Stop,
                        });
                    }
                    (glutin::event::VirtualKeyCode::A, ElementState::Pressed) => {
                        game.player_command(PlayerCommand::Rotation {
                            direction: RotationDirection::Left,
                            action: MovementAction::Start,
                        });
                    }
                    (glutin::event::VirtualKeyCode::A, ElementState::Released) => {
                        game.player_command(PlayerCommand::Rotation {
                            direction: RotationDirection::Left,
                            action: MovementAction::Stop,
                        });
                    }
                    (glutin::event::VirtualKeyCode::D, ElementState::Pressed) => {
                        game.player_command(PlayerCommand::Rotation {
                            direction: RotationDirection::Right,
                            action: MovementAction::Start,
                        });
                    }
                    (glutin::event::VirtualKeyCode::D, ElementState::Released) => {
                        game.player_command(PlayerCommand::Rotation {
                            direction: RotationDirection::Right,
                            action: MovementAction::Stop,
                        });
                    }
                    (glutin::event::VirtualKeyCode::Space, ElementState::Pressed) => {
                        game.player_command(PlayerCommand::Shoot);
                    }
                    _ => (),
                }
            }
        }
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            println!("The close button was pressed; stopping");
            *control_flow = ControlFlow::Exit
        }
        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
        // dispatched any events. This is ideal for games and similar applications.
        _ => *control_flow = ControlFlow::Poll,
    });
}
