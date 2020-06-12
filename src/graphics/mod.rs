use gl::types::{GLenum, GLfloat, GLint, GLsizei, GLsizeiptr, GLuint};
use nalgebra_glm as glm;
use std::convert::TryInto;
use std::ffi::{c_void, CStr, CString};
use std::ptr::{null, null_mut};

#[derive(Debug, Clone)]
pub struct OpenGLError {
    msg: CString,
}

impl std::fmt::Display for OpenGLError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "OpenGLError: {}", self.msg.to_string_lossy())
    }
}

impl std::error::Error for OpenGLError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[derive(Debug, Clone)]
pub enum RenderComponent {
    DrawArrays {
        vao: GLuint,
        first: GLint,
        count: GLsizei,
        height: f32,
        width: f32,
        mode: GLenum,
    },
    DrawElements {
        vao: GLuint,
        count: GLsizei,
        height: f32,
        width: f32,
    },
}

impl RenderComponent {
    ///  # Safety
    ///  this is unsafe because every opengl function operates over an
    ///  invisible mutable state
    pub unsafe fn new_triangle(width: f32, height: f32) -> RenderComponent {
        let vao = {
            let mut vao = std::mem::MaybeUninit::<GLuint>::uninit();
            gl::GenVertexArrays(1, vao.as_mut_ptr());
            vao.assume_init()
        };

        gl::BindVertexArray(vao);

        let vertices = [0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];

        let mut buffer_id = std::mem::MaybeUninit::<GLuint>::uninit();
        gl::GenBuffers(1, buffer_id.as_mut_ptr());
        let buffer_id = buffer_id.assume_init();
        gl::BindBuffer(gl::ARRAY_BUFFER, buffer_id);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            std::mem::size_of_val(&vertices) as GLsizeiptr,
            &(vertices[0]) as *const f32 as *const c_void,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,                                     //location = 0
            3,                                     // size of the vertex attribute (vec3)
            gl::FLOAT,                             //type
            gl::FALSE,                             //normalization
            3 * std::mem::size_of::<f32>() as i32, //stride? size of each vertex (attribute)
            null_mut::<std::ffi::c_void>(),        // offset where the vertex attribute starts
        );
        gl::EnableVertexAttribArray(0);

        RenderComponent::DrawArrays {
            vao,
            first: 0,
            count: 3,
            mode: gl::TRIANGLES,
            width,
            height,
        }
    }

    ///  # Safety
    ///  this is unsafe because every opengl function operates over an
    ///  invisible mutable state
    pub unsafe fn new_square(width: f32, height: f32) -> RenderComponent {
        let vao = {
            let mut vao = std::mem::MaybeUninit::<GLuint>::uninit();
            gl::GenVertexArrays(1, vao.as_mut_ptr());
            vao.assume_init()
        };

        gl::BindVertexArray(vao);

        let vertices = [
            0f32, 0.0, 0.0, // bottom left
            1.0, 0.0, 0.0, // bottom right
            1.0, 1.0, 0.0, // top right
            0.0, 1.0, 0.0, // top left
        ];

        let vbo_buffer_id = {
            let mut vbo_buffer_id = std::mem::MaybeUninit::<GLuint>::uninit();
            gl::GenBuffers(1, vbo_buffer_id.as_mut_ptr());
            vbo_buffer_id.assume_init()
        };

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo_buffer_id);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            std::mem::size_of_val(&vertices) as GLsizeiptr,
            &(vertices[0]) as *const f32 as *const c_void,
            gl::STATIC_DRAW,
        );

        let indices = [0u32, 1, 2, 0, 3, 2];

        let ebo_buffer_id = {
            let mut ebo_buffer_id = std::mem::MaybeUninit::<GLuint>::uninit();
            gl::GenBuffers(1, ebo_buffer_id.as_mut_ptr());
            ebo_buffer_id.assume_init()
        };

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo_buffer_id);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            std::mem::size_of_val(&indices) as GLsizeiptr,
            &(indices[0]) as *const u32 as *const c_void,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,                                     //location = 0
            3,                                     // size of the vertex attribute (vec3)
            gl::FLOAT,                             //type
            gl::FALSE,                             //normalization
            3 * std::mem::size_of::<f32>() as i32, //stride? size of each vertex (attribute)
            null_mut::<std::ffi::c_void>(),        // offset where the vertex attribute starts
        );

        gl::EnableVertexAttribArray(0);

        RenderComponent::DrawElements {
            vao,
            count: 6,
            width,
            height,
        }
    }

    ///  # Safety
    ///  this is unsafe because every opengl function operates over an
    ///  invisible mutable state
    pub unsafe fn new_circle(r: f32) -> RenderComponent {
        let vao = {
            let mut vao = std::mem::MaybeUninit::<GLuint>::uninit();
            gl::GenVertexArrays(1, vao.as_mut_ptr());
            assert_eq!(gl::GetError(), gl::NO_ERROR);
            vao.assume_init()
        };

        gl::BindVertexArray(vao);

        let segments: u16 = (10.0 * r.sqrt()).floor() as u16;
        let vertices = gen_circle(0.0, 0.0, r, segments);

        let buffer_id = {
            let mut buffer_id = std::mem::MaybeUninit::<GLuint>::uninit();
            gl::GenBuffers(1, buffer_id.as_mut_ptr());
            buffer_id.assume_init()
        };

        gl::BindBuffer(gl::ARRAY_BUFFER, buffer_id);

        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
            &(vertices[0]) as *const f32 as *const c_void,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,                                     //location = 0
            3,                                     // size of the vertex attribute (vec3)
            gl::FLOAT,                             //type
            gl::FALSE,                             //normalization
            3 * std::mem::size_of::<f32>() as i32, //stride? size of each vertex (attribute)
            null_mut::<std::ffi::c_void>(),        // offset where the vertex attribute starts
        );

        gl::EnableVertexAttribArray(0);

        // HACK: for some reason my circles end up as ellipses, so I adjust the scale later to 0.8
        // TODO: fix the hack^
        RenderComponent::DrawArrays {
            vao,
            first: 0,
            count: segments as GLint,
            mode: gl::TRIANGLE_FAN,
            width: 0.8,
            height: 1.0,
        }
    }

    ///  # Safety
    ///  this is unsafe because every opengl function operates over an
    ///  invisible mutable state
    pub unsafe fn new_shooter(r1: f32, r2: f32) -> RenderComponent {
        let vao = {
            let mut vao = std::mem::MaybeUninit::<GLuint>::uninit();
            gl::GenVertexArrays(1, vao.as_mut_ptr());
            assert_eq!(gl::GetError(), gl::NO_ERROR);
            vao.assume_init()
        };

        gl::BindVertexArray(vao);

        let segments1: u16 = (10.0 * r1.sqrt()).floor() as u16;
        let segments2: u16 = (10.0 * r2.sqrt()).floor() as u16;

        let vertices_main_body = gen_circle(0.0, 0.0, r1, segments1);
        let vertices_aim_indicator = gen_circle(r1 + 1.5, 0.0, r2, segments2);

        let vertices: Vec<f32> = vertices_main_body
            .iter()
            .cloned()
            .chain(vertices_aim_indicator.iter().cloned())
            .collect();

        let buffer_id = {
            let mut buffer_id = std::mem::MaybeUninit::<GLuint>::uninit();
            gl::GenBuffers(1, buffer_id.as_mut_ptr());
            buffer_id.assume_init()
        };

        gl::BindBuffer(gl::ARRAY_BUFFER, buffer_id);

        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
            &(vertices[0]) as *const f32 as *const c_void,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,                                     //location = 0
            3,                                     // size of the vertex attribute (vec3)
            gl::FLOAT,                             //type
            gl::FALSE,                             //normalization
            3 * std::mem::size_of::<f32>() as i32, //stride? size of each vertex (attribute)
            null_mut::<std::ffi::c_void>(),        // offset where the vertex attribute starts
        );

        gl::EnableVertexAttribArray(0);

        // HACK: for some reason my circles end up as ellipses, so I adjust the scale later to 0.8
        // TODO: fix the hack^
        RenderComponent::DrawArrays {
            vao,
            first: 0,
            count: (segments1 + segments2) as GLint,
            mode: gl::TRIANGLE_FAN,
            width: 0.8,
            height: 1.0,
        }
    }

    pub fn draw(&self, program: &mut Program) {
        match self {
            Self::DrawArrays {
                vao,
                first,
                count,
                width,
                height,
                mode,
            } => unsafe {
                gl::BindVertexArray(*vao);

                // FIXME: there must be a way construct this easier (otherwise, I could store a lazy_static?)
                let identity = glm::mat3_to_mat4(&glm::mat3(
                    1f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 1f32,
                ));

                let scale = glm::scale(&identity, &glm::vec3(*width, *height, 1.0));

                program.set_scale(glm::value_ptr(&scale));

                gl::DrawArrays(*mode, *first, *count);
            },
            Self::DrawElements {
                vao,
                count,
                height,
                width,
            } => unsafe {
                gl::BindVertexArray(*vao);

                // FIXME: same comment that above
                let identity = glm::mat3_to_mat4(&glm::mat3(
                    1f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 1f32,
                ));

                let scale = glm::scale(&identity, &glm::vec3(*width, *height, 1.0));

                program.set_scale(glm::value_ptr(&scale));

                gl::DrawElements(
                    gl::TRIANGLES,
                    *count,
                    gl::UNSIGNED_INT,
                    null::<std::ffi::c_void>(),
                );
            },
        }
    }
}

pub unsafe fn create_shader<T: AsRef<CStr>>(
    text: T,
    shader_type: GLenum,
) -> Result<GLuint, OpenGLError> {
    let vertex_shader = gl::CreateShader(shader_type);

    gl::ShaderSource(
        vertex_shader,
        1,
        &(text.as_ref().as_ptr() as *const u8 as *const i8) as *const *const i8,
        std::ptr::null(),
    );

    gl::CompileShader(vertex_shader);

    let mut success = std::mem::MaybeUninit::<i32>::uninit();
    let mut buffer = [0u8; 512];
    gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, success.as_mut_ptr());

    let success = success.assume_init();

    if success != gl::TRUE.try_into().unwrap() {
        // TODO, the third argument tells the actual length of the error message, collect it entirely
        gl::GetShaderInfoLog(
            vertex_shader,
            512,
            std::ptr::null_mut(),
            buffer.as_mut_ptr() as *mut u8 as *mut i8,
        );

        let c_string = CStr::from_bytes_with_nul(&buffer).expect("Invalid ShaderInfoLog");
        return Err(OpenGLError {
            msg: c_string.to_owned(),
        });
    }

    Ok(vertex_shader)
}

pub struct Program {
    id: GLuint,
}

impl Program {
    pub unsafe fn new() -> Result<Program, OpenGLError> {
        let vertex_shader_src = {
            let src: Vec<u8> = include_bytes!("shaders/vertex.vert").as_ref().into();
            CString::new(src).unwrap()
        };

        let fragment_shader_src = {
            let src: Vec<u8> = include_bytes!("shaders/fragment.fs").as_ref().into();
            CString::new(src).unwrap()
        };

        let vertex = create_shader(vertex_shader_src, gl::VERTEX_SHADER)?;
        let fragment = create_shader(fragment_shader_src, gl::FRAGMENT_SHADER)?;

        let id = gl::CreateProgram();
        gl::AttachShader(id, vertex);
        gl::AttachShader(id, fragment);
        gl::LinkProgram(id);

        let success = {
            let mut success = std::mem::MaybeUninit::<i32>::uninit();
            gl::GetProgramiv(id, gl::LINK_STATUS, success.as_mut_ptr());
            success.assume_init()
        };

        if success != gl::TRUE.into() {
            let mut buffer = [0u8; 512];
            gl::GetProgramInfoLog(
                id,
                500,
                std::ptr::null_mut(),
                buffer.as_mut_ptr() as *mut u8 as *mut i8,
            );
            let c_string = CStr::from_bytes_with_nul(&buffer).expect("invalid ProgramInfoLog");

            return Err(OpenGLError {
                msg: c_string.to_owned(),
            });
        }

        gl::DeleteShader(vertex);
        gl::DeleteShader(fragment);
        Ok(Program { id })
    }

    pub unsafe fn set_active(&self) {
        gl::UseProgram(self.id);
    }

    pub unsafe fn set_uniform_float(&mut self, name: impl AsRef<CStr>, value: GLfloat) {
        let location = gl::GetUniformLocation(self.id, name.as_ref().as_ptr());
        gl::Uniform1f(location, value);
    }

    pub fn set_translation(&mut self, value: &[f32]) {
        unsafe {
            self.set_uniform_matrix_4fv(
                CStr::from_bytes_with_nul(b"translation\0").unwrap(),
                value,
            );
        }
    }

    pub fn set_projection(&mut self, value: &[f32]) {
        unsafe {
            self.set_uniform_matrix_4fv(CStr::from_bytes_with_nul(b"projection\0").unwrap(), value);
        }
    }

    pub fn set_scale(&mut self, value: &[f32]) {
        unsafe {
            self.set_uniform_matrix_4fv(CStr::from_bytes_with_nul(b"scale\0").unwrap(), value);
        }
    }

    pub fn set_rotation(&mut self, value: &[f32]) {
        unsafe {
            self.set_uniform_matrix_4fv(CStr::from_bytes_with_nul(b"rotation\0").unwrap(), value);
        }
    }

    pub fn set_color(&mut self, x: f32, y: f32, z: f32) {
        unsafe {
            let location = gl::GetUniformLocation(
                self.id,
                CStr::from_bytes_with_nul(b"color\0").unwrap().as_ptr(),
            );

            gl::Uniform3f(location, x, y, z);
        }
    }

    unsafe fn set_uniform_matrix_4fv(&mut self, name: impl AsRef<CStr>, value: &[f32]) {
        let location = gl::GetUniformLocation(self.id, name.as_ref().as_ptr());
        gl::UniformMatrix4fv(location, 1, gl::FALSE, value.as_ptr());
    }
}

// taken from: http://slabode.exofire.net/circle_draw.shtml
// xxx: write with vectors?
// or the other version?
fn gen_circle(cx: f32, cy: f32, r: f32, segments: u16) -> Box<[f32]> {
    // TODO: remove try_from... use f64? or set a max cap on segmefts
    let theta = (2.0 * std::f32::consts::PI) / f32::from(segments);
    let tangential_factor = theta.tan();
    let radial_factor = theta.cos();

    let mut x = r;
    let mut y = 0.0;

    // XXX: write this functionally and measure if it takes the same?
    let final_length: usize = (segments * 3) as usize;
    let mut return_value = Vec::with_capacity(final_length);

    for _ in 0..segments {
        return_value.push(x + cx);
        return_value.push(y + cy);
        return_value.push(0.0);

        // get an orthogonal vector to (x, y)
        let tx = -y;
        let ty = x;

        x += tx * tangential_factor;
        y += ty * tangential_factor;

        x *= radial_factor;
        y *= radial_factor;
    }

    return_value.into_boxed_slice()
}
