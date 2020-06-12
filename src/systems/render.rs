use crate::graphics::{OpenGLError, Program};
use crate::{Arena, ComponentManager, OrientationComponent, PositionComponent};
use crate::{X_MAX, Y_MAX};
use nalgebra_glm as glm;

pub struct RenderSystem {
    program: Program,
}

impl RenderSystem {
    pub fn new() -> Result<Self, OpenGLError> {
        let mut program = unsafe { Program::new()? };

        unsafe { program.set_active() };

        let projection = glm::ortho(0.0f32, X_MAX, 0.0f32, Y_MAX, 0.0f32, 1.0f32);

        program.set_projection(glm::value_ptr(&projection));

        Ok(RenderSystem { program })
    }

    pub fn render(&mut self, arena: &Arena, components: &ComponentManager) {
        let identity = glm::mat3_to_mat4(&glm::mat3(
            1f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 1f32,
        ));

        let arena_width = crate::X_MAX * arena.percent;
        let arena_height = crate::Y_MAX * arena.percent;

        let x_thresh = (crate::X_MAX - arena_width) / 2.0;
        let y_thresh = (crate::Y_MAX - arena_height) / 2.0;

        let arena = unsafe {
            gl::ClearColor(0.1, 0.2, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            crate::RenderComponent::new_square(arena_width, arena_height)
        };

        self.program.set_rotation(glm::value_ptr(&identity));
        let translation = glm::translate(&identity, &glm::vec3(x_thresh, y_thresh, 0f32));
        self.program.set_translation(glm::value_ptr(&translation));

        self.program.set_color(0.2, 0.1, 0.8);

        arena.draw(&mut self.program);

        for (index, render) in components.render.iter().enumerate() {
            if let Some(render) = render {
                let PositionComponent { x, y } = components.position[index]
                    .as_ref()
                    .expect("render component doesn't have a position");

                let translation = glm::translate(&identity, &glm::vec3(*x, *y, 0f32));

                let rotation = match components
                    .orientation
                    .get(index)
                    .map(|inner| inner.as_ref())
                    .flatten()
                    .as_ref()
                {
                    Some(OrientationComponent { angle }) => {
                        glm::rotate(&identity, *angle, &glm::vec3(0.0, 0.0, 1.0))
                    }
                    None => identity,
                };

                self.program.set_rotation(glm::value_ptr(&rotation));

                self.program.set_translation(glm::value_ptr(&translation));

                self.program.set_color(1.0, 0.5, 0.2);

                render.draw(&mut self.program);
            }
        }
    }
}
