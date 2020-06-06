extern crate simple_platformer;
use glutin::{event::Event, event::WindowEvent, event_loop::ControlFlow, Api, GlRequest};
use simple_platformer::*;

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

    let mut dpi = gl_current.window().hidpi_factor();

    let mut render_system = systems::RenderSystem::new().unwrap();
    let physics_system = systems::PhysicsSystem::new();
    let collision_system = systems::CollisionSystem::new();
    let mut logic_system = systems::LogicSystem::new();

    let mut entity_manager = EntityManager::new();
    let mut component_manager = ComponentManager::new();

    let player_entity = entity_manager.next_entity();
    component_manager.set_position_component(
        player_entity,
        PositionComponent::new_wrapping(0.0f32, 0.0f32),
    );
    // component_manager.set_render_component(triangle_entity, unsafe {
    //     RenderComponent::new_triangle(100f32)
    // });
    // component_manager.set_render_component(triangle_entity, unsafe {
    //     RenderComponent::new_square(100f32, 200.0)
    // });
    let player_size = 30.0;
    component_manager.set_render_component(player_entity, unsafe {
        RenderComponent::new_shooter(player_size, 5.0)
    });
    component_manager.set_collision_component(player_entity, CollisionComponent::new(player_size));

    component_manager.set_body_component(player_entity, BodyComponent::new(10.0, 0.4));
    component_manager.set_orientation_component(player_entity, OrientationComponent::new(0.0));

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
        Event::EventsCleared => {
            // Application update code.

            let new_instant = std::time::Instant::now();
            let dt = (new_instant - last_instant).as_secs_f64();
            last_instant = new_instant;
            physics_system.run(dt, &mut component_manager);

            collision_system.run(&mut component_manager, |a, b| {
                logic_system.push_event(LogicMessage::Collision(a, b))
            });

            logic_system.run(&mut entity_manager, &mut component_manager);

            for entity in entity_manager.iter() {
                if entity != player_entity {
                    let position = component_manager.get_position_component(entity).unwrap();
                    println!("{:?} - {:?}", entity, position);
                }
            }

            // Queue a RedrawRequested event.
            gl_current.window().request_redraw();
        }
        Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } => {
            // Redraw the application.
            //
            // It's preferrable to render in this event rather than in EventsCleared, since
            // rendering in here allows the program to gracefully handle redraws requested
            // by the OS.
            //
            render_system.render(&component_manager);

            gl_current.swap_buffers().unwrap();
        }
        Event::WindowEvent {
            event: WindowEvent::HiDpiFactorChanged(new_dpi),
            ..
        } => dpi = new_dpi,
        Event::WindowEvent {
            event: WindowEvent::Resized(size),
            ..
        } => {
            gl_current.resize(size.to_physical(dpi));
        }
        Event::WindowEvent {
            event: WindowEvent::KeyboardInput { input, .. },
            ..
        } => {
            if let Some(key_code) = input.virtual_keycode {
                let force_to_apply = 500_000.0;

                use glutin::event::ElementState;
                match (key_code, input.state) {
                    (glutin::event::VirtualKeyCode::Escape, ElementState::Pressed) => {
                        println!("The escape key was pressed; stopping");
                        *control_flow = ControlFlow::Exit;
                    }
                    (glutin::event::VirtualKeyCode::W, ElementState::Pressed) => {
                        let orientation = *component_manager
                            .get_orientation_component(player_entity)
                            .expect("player has no orientation");

                        component_manager.update_body_component(player_entity, |body| {
                            body.apply_force_x(f64::from(orientation.angle.cos()) * force_to_apply);
                            body.apply_force_y(f64::from(orientation.angle.sin()) * force_to_apply);
                        });
                    }
                    (glutin::event::VirtualKeyCode::S, ElementState::Pressed) => {
                        let orientation = *component_manager
                            .get_orientation_component(player_entity)
                            .expect("player has no orientation");

                        component_manager.update_body_component(player_entity, |body| {
                            body.apply_force_x(
                                -f64::from(orientation.angle.cos()) * force_to_apply,
                            );
                            body.apply_force_y(
                                -f64::from(orientation.angle.sin()) * force_to_apply,
                            );
                        });
                    }
                    (glutin::event::VirtualKeyCode::A, ElementState::Pressed) => {
                        // player_orientation += std::f32::consts::PI / 12.0;
                        component_manager.update_orientation_component(
                            player_entity,
                            |component| {
                                component.angle += std::f32::consts::PI / 80.0;
                            },
                        );
                    }
                    (glutin::event::VirtualKeyCode::D, ElementState::Pressed) => {
                        component_manager.update_orientation_component(
                            player_entity,
                            |component| {
                                component.angle -= std::f32::consts::PI / 80.0;
                            },
                        );
                    }
                    (glutin::event::VirtualKeyCode::Space, ElementState::Pressed) => {
                        if let Some(OrientationComponent { angle }) =
                            component_manager.get_orientation_component(player_entity)
                        {
                            logic_system.push_event(LogicMessage::Shoot {
                                shooter: player_entity,
                                orientation: *angle,
                            });
                        }
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
