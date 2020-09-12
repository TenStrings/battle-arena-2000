use battle_arena_2000::*;
use futures::{SinkExt, StreamExt};
use glutin::{event::Event, event::WindowEvent, event_loop::ControlFlow, Api, GlRequest};
use log::{debug, info};
use network::Packet;
use std::time::Duration;

#[async_std::main]
async fn main() -> Result<(), ()> {
    env_logger::init();

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

    let mut server = network::Client::<Box<[u8]>>::new("0.0.0.0:9998", "127.0.0.1:9999")
        .await
        .unwrap();

    let server_connection = server.connection();

    async_std::task::spawn(async move {
        let mut interval = async_std::stream::interval(Duration::from_millis(100));
        let mut last_instant = std::time::Instant::now();
        while let Some(_) = interval.next().await {
            let new_instant = std::time::Instant::now();

            let dt = new_instant - last_instant;
            last_instant = new_instant;
            let failed_packets = server_connection.lock().await.update(dt);

            for seq in failed_packets {
                info!("packet with SeqNumber {} failed to be sent", seq);
            }
        }
    });

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            gl_current.window().request_redraw();
        }
        Event::RedrawRequested { .. } => {
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
                        let cmd = PlayerCommand::Movement {
                            direction: MovementDirection::Up,
                            action: MovementAction::Start,
                        };

                        debug!("sending {:?}", cmd);

                        let mut packet = Packet::new_boxed(3);
                        cmd.to_bytes(packet.payload_mut());
                        async_std::task::block_on(server.send(packet)).unwrap();
                    }
                    (glutin::event::VirtualKeyCode::W, ElementState::Released) => {
                        let cmd = PlayerCommand::Movement {
                            direction: MovementDirection::Up,
                            action: MovementAction::Stop,
                        };

                        debug!("sending {:?}", cmd);

                        let mut packet = Packet::new_boxed(3);
                        cmd.to_bytes(packet.payload_mut());
                        async_std::task::block_on(server.send(packet)).unwrap();
                    }
                    (glutin::event::VirtualKeyCode::S, ElementState::Pressed) => {
                        let cmd = PlayerCommand::Movement {
                            direction: MovementDirection::Down,
                            action: MovementAction::Start,
                        };

                        debug!("sending {:?}", cmd);

                        let mut packet = Packet::new_boxed(3);
                        cmd.to_bytes(packet.payload_mut());
                        async_std::task::block_on(server.send(packet)).unwrap();
                    }
                    (glutin::event::VirtualKeyCode::S, ElementState::Released) => {
                        let cmd = PlayerCommand::Movement {
                            direction: MovementDirection::Down,
                            action: MovementAction::Stop,
                        };

                        debug!("sending {:?}", cmd);

                        let mut packet = Packet::new_boxed(3);
                        cmd.to_bytes(packet.payload_mut());
                        async_std::task::block_on(server.send(packet)).unwrap();
                    }
                    (glutin::event::VirtualKeyCode::A, ElementState::Pressed) => {
                        let cmd = PlayerCommand::Rotation {
                            direction: RotationDirection::Left,
                            action: MovementAction::Start,
                        };

                        debug!("sending {:?}", cmd);

                        let mut packet = Packet::new_boxed(3);
                        cmd.to_bytes(packet.payload_mut());
                        async_std::task::block_on(server.send(packet)).unwrap();
                    }
                    (glutin::event::VirtualKeyCode::A, ElementState::Released) => {
                        let cmd = PlayerCommand::Rotation {
                            direction: RotationDirection::Left,
                            action: MovementAction::Stop,
                        };

                        debug!("sending {:?}", cmd);

                        let mut packet = Packet::new_boxed(3);
                        cmd.to_bytes(packet.payload_mut());
                        debug!("blocking on send");
                        async_std::task::block_on(server.send(packet)).unwrap();
                        debug!("unblocked");
                    }
                    (glutin::event::VirtualKeyCode::D, ElementState::Pressed) => {
                        let cmd = PlayerCommand::Rotation {
                            direction: RotationDirection::Right,
                            action: MovementAction::Start,
                        };

                        debug!("sending {:?}", cmd);

                        let mut packet = Packet::new_boxed(3);
                        cmd.to_bytes(packet.payload_mut());
                        async_std::task::block_on(server.send(packet)).unwrap();
                    }
                    (glutin::event::VirtualKeyCode::D, ElementState::Released) => {
                        let cmd = PlayerCommand::Rotation {
                            direction: RotationDirection::Right,
                            action: MovementAction::Stop,
                        };

                        debug!("sending {:?}", cmd);

                        let mut packet = Packet::new_boxed(3);
                        cmd.to_bytes(packet.payload_mut());
                        async_std::task::block_on(server.send(packet)).unwrap();
                    }
                    (glutin::event::VirtualKeyCode::Space, ElementState::Pressed) => {
                        let cmd = PlayerCommand::Shoot;

                        debug!("sending {:?}", cmd);

                        let mut packet = Packet::new_boxed(3);
                        cmd.to_bytes(packet.payload_mut());
                        async_std::task::block_on(server.send(packet)).unwrap();
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
        _ => *control_flow = ControlFlow::Poll,
    });
}
