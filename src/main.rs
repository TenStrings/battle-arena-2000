use gl::types::{GLsizeiptr, GLuint};
use glutin::{event::Event, event::WindowEvent, event_loop::ControlFlow, Api, GlRequest};
use std::ffi::{c_void, CStr};

fn main() {
    let event_loop = glutin::event_loop::EventLoop::new();
    let window_builder = glutin::window::WindowBuilder::new()
        .with_title("Hello world!")
        .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));

    let windowed_context = glutin::ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .build_windowed(window_builder, &event_loop)
        .unwrap();

    let gl_current = unsafe { windowed_context.make_current().unwrap() };

    unsafe {
        gl::load_with(|symbol| gl_current.get_proc_address(symbol) as *const _);
        gl::Viewport(0, 0, 1024, 768)
    };

    let mut dpi = gl_current.window().hidpi_factor();

    let mut color: [f32; 4] = [0.3, 0.4, 0.1, 1.0];

    let vertices = [-0.5f32, -0.5, 0.0, 0.5, -0.5, 0.0, 0.0, 0.5, 0.0];

    let vertex_shader_src = std::ffi::CString::new(
        r#"
        #version 330 core
        layout (location = 0) in vec3 aPos;

        void main()
        {
            gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
        }
    "#,
    )
    .unwrap();

    unsafe {
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(
            vertex_shader,
            1,
            &(vertex_shader_src.as_ptr() as *const u8 as *const i8) as *const *const i8,
            std::ptr::null(),
        );
        gl::CompileShader(vertex_shader);

        let mut success: i32 = std::mem::uninitialized();
        let mut buffer = [0u8; 512];
        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success as *mut i32);
        if success != 0 {
            gl::GetShaderInfoLog(
                vertex_shader,
                512,
                std::ptr::null_mut(),
                buffer.as_mut_ptr() as *mut u8 as *mut i8,
            );
            let c_string = CStr::from_bytes_with_nul(&buffer).unwrap();
            println!("{}", c_string.to_str().unwrap());
        }
    }

    let fragment_shader_src = std::ffi::CString::new(
        r#"
        #version 330 core
        out vec4 FragColor;

        void main()
        {
            FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
        } 
    "#,
    )
    .unwrap();

    unsafe {
        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(
            fragment_shader,
            1,
            &(fragment_shader_src.as_ptr() as *const u8 as *const i8) as *const *const i8,
            std::ptr::null(),
        );
        gl::CompileShader(fragment_shader);

        let mut success: i32 = std::mem::uninitialized();
        let mut buffer = [0u8; 512];
        gl::GetShaderiv(
            fragment_shader,
            gl::COMPILE_STATUS,
            &mut success as *mut i32,
        );
        if success != 0 {
            gl::GetShaderInfoLog(
                fragment_shader,
                512,
                std::ptr::null_mut(),
                buffer.as_mut_ptr() as *mut u8 as *mut i8,
            );
            let c_string = CStr::from_bytes_with_nul(&buffer).unwrap();
            println!("{}", c_string.to_str().unwrap());
        }
    }

    unsafe {
        let mut buffer_id: gl::types::GLuint = std::mem::uninitialized();
        gl::GenBuffers(1, &mut buffer_id as *mut GLuint);
        gl::BindBuffer(gl::ARRAY_BUFFER, buffer_id);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            std::mem::size_of_val(&vertices) as GLsizeiptr,
            &(vertices[0]) as *const f32 as *const c_void,
            gl::STATIC_DRAW,
        );
    };

    event_loop.run(move |event, _, control_flow| match event {
        Event::EventsCleared => {
            // Application update code.

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
            let [r, g, b, a] = color;
            unsafe {
                gl::ClearColor(r, g, b, a);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }
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
                match key_code {
                    glutin::event::VirtualKeyCode::Escape => {
                        println!("The escape key was pressed; stopping");
                        *control_flow = ControlFlow::Exit;
                    }
                    glutin::event::VirtualKeyCode::C => {
                        println!("C key pressed; changing color");
                        color = [0.5, 0.5, 0.5, 1.0];
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
        // ControlFlow::Wait pauses the event loop if no events are available to process.
        // This is ideal for non-game applications that only update in response to user
        // input, and uses significantly less power/CPU time than ControlFlow::Poll.
        // _ => *control_flow = ControlFlow::Wait,
    });
}
