use crate::graphics::{OpenGLError, Program};
use crate::{ComponentManager, PositionComponent};
use crate::{X_MAX, Y_MAX};
use nalgebra_glm as glm;

pub struct RenderSystem {
    program: Program,
}

impl RenderSystem {
    pub fn new() -> Result<Self, OpenGLError> {
        let mut program = unsafe { Program::new()? };

        unsafe { program.set_active() };

        let projection = glm::ortho(0.0f32, X_MAX, 0.0f32, Y_MAX, 0.0f32, 0.1f32);

        program.set_projection(glm::value_ptr(&projection));

        Ok(RenderSystem { program })
    }

    pub fn render(&mut self, components: &ComponentManager) {
        unsafe {
            gl::ClearColor(0.1, 0.2, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        for (index, render) in components.render.iter().enumerate() {
            if let Some(render) = render {
                let PositionComponent { x, y } = components.position[index]
                    .as_ref()
                    .expect("render component doesn't have a position");

                let identity = glm::mat3_to_mat4(&glm::mat3(
                    1f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 1f32,
                ));

                let translation = glm::translate(&identity, &glm::vec3(*x, *y, 0f32));

                self.program.set_translation(glm::value_ptr(&translation));

                render.draw(&mut self.program);
            }
        }
    }
}
