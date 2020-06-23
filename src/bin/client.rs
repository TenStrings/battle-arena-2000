extern crate simple_platformer;
#[macro_use]
extern crate log;
use glutin::{event::Event, event::WindowEvent, event_loop::ControlFlow, Api, GlRequest};
use log::{error, info};
use simple_platformer::*;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};

use clap::{App, Arg};

fn main() -> Result<(), ()> {
    env_logger::init();

    let matches = App::new("battle arena 2000")
        .version("0.0.1-alpha")
        .author("Enzo Cioppettini <hi@enzocioppettini.com>")
        .about("Push people out")
        .arg(
            Arg::new("server address")
                .short('s')
                .long("server")
                .takes_value(true),
        )
        .get_matches();

    let server_address = matches
        .value_of("server address")
        .unwrap_or("127.0.0.1:6666");

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

    let server = TcpStream::connect(server_address).expect("couldn't connect to server");

    info!("connected to server");

    let mut client = client::Client::new(server);

    event_loop.run(move |event, _, control_flow| match event {
        Event::EventsCleared => {
            client.run();
            gl_current.window().request_redraw();
        }
        Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } => {
            client.render();
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
                use glutin::event::ElementState;
                match (key_code, input.state) {
                    (glutin::event::VirtualKeyCode::Escape, ElementState::Pressed) => {
                        println!("The escape key was pressed; stopping");
                        *control_flow = ControlFlow::Exit;
                    }
                    (glutin::event::VirtualKeyCode::W, ElementState::Pressed) => {
                        client.player_command(PlayerCommand::Movement {
                            direction: MovementDirection::Up,
                            action: MovementAction::Start,
                        });
                    }
                    (glutin::event::VirtualKeyCode::W, ElementState::Released) => {
                        client.player_command(PlayerCommand::Movement {
                            direction: MovementDirection::Up,
                            action: MovementAction::Stop,
                        });
                    }
                    (glutin::event::VirtualKeyCode::S, ElementState::Pressed) => {
                        client.player_command(PlayerCommand::Movement {
                            direction: MovementDirection::Down,
                            action: MovementAction::Start,
                        });
                    }
                    (glutin::event::VirtualKeyCode::S, ElementState::Released) => {
                        client.player_command(PlayerCommand::Movement {
                            direction: MovementDirection::Down,
                            action: MovementAction::Stop,
                        });
                    }
                    (glutin::event::VirtualKeyCode::A, ElementState::Pressed) => {
                        client.player_command(PlayerCommand::Rotation {
                            direction: RotationDirection::Left,
                            action: MovementAction::Start,
                        });
                    }
                    (glutin::event::VirtualKeyCode::A, ElementState::Released) => {
                        client.player_command(PlayerCommand::Rotation {
                            direction: RotationDirection::Left,
                            action: MovementAction::Stop,
                        });
                    }
                    (glutin::event::VirtualKeyCode::D, ElementState::Pressed) => {
                        client.player_command(PlayerCommand::Rotation {
                            direction: RotationDirection::Right,
                            action: MovementAction::Start,
                        });
                    }
                    (glutin::event::VirtualKeyCode::D, ElementState::Released) => {
                        client.player_command(PlayerCommand::Rotation {
                            direction: RotationDirection::Right,
                            action: MovementAction::Stop,
                        });
                    }
                    (glutin::event::VirtualKeyCode::Space, ElementState::Pressed) => {
                        client.player_command(PlayerCommand::Shoot);
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
        // dispatched any events. This is ideal for clients and similar applications.
        _ => *control_flow = ControlFlow::Poll,
    });
}
