use battle_arena_2000::*;
use futures::StreamExt;
use glutin::{
    event::Event, event::WindowEvent, event_loop::ControlFlow, event_loop::EventLoop, Api,
    GlRequest,
};
use log::{debug, info};
use std::convert::TryInto;
use std::time::Duration;

#[async_std::main]
async fn main() -> Result<(), ()> {
    env_logger::init();

    let event_loop: EventLoop<PlayerCommand> = glutin::event_loop::EventLoop::with_user_event();
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

    let mut last_instant = std::time::Instant::now();

    let mut player1 = network::Client::<Box<[u8]>>::new("0.0.0.0:9999", "127.0.0.1:9998")
        .await
        .unwrap();

    let player1_connection = player1.connection();

    let proxy = event_loop.create_proxy();
    async_std::task::spawn(async move {
        while let Some(packet) = player1.next().await {
            let cmd: [u8; 3] = packet.payload().try_into().unwrap();
            let player_command = PlayerCommand::from_bytes(&cmd);

            debug!("received player command {:?}", player_command);
            proxy.send_event(player_command).unwrap();
        }
    });

    async_std::task::spawn(async move {
        let mut interval = async_std::stream::interval(Duration::from_millis(100));
        let mut last_instant = std::time::Instant::now();
        while let Some(_) = interval.next().await {
            let new_instant = std::time::Instant::now();

            let dt = new_instant - last_instant;
            last_instant = new_instant;
            let failed_packets = player1_connection.lock().await.update(dt);

            for seq in failed_packets {
                info!("packet with SeqNumber {} failed to be sent", seq);
            }
        }
    });

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            let new_instant = std::time::Instant::now();
            let dt = new_instant - last_instant;
            last_instant = new_instant;

            game.update_state(dt);

            gl_current.window().request_redraw();
        }
        Event::RedrawRequested { .. } => {
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
        Event::UserEvent(cmd) => game.player_command(cmd),
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
