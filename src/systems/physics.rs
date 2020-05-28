use crate::ComponentManager;
use crate::PositionComponent;
use nalgebra_glm as glm;

pub struct PhysicsSystem {}

#[derive(Clone, Debug)]
pub struct BodyComponent {
    net_force: glm::TVec2<f64>,
    acceleration: glm::TVec2<f64>,
    velocity: glm::TVec2<f64>,
    mass: f64,
}

impl PhysicsSystem {
    pub fn new() -> PhysicsSystem {
        PhysicsSystem {}
    }

    pub fn run(&self, dt: f64, components: &mut ComponentManager) {
        for (index, body) in components.body.iter_mut().enumerate() {
            if let Some(body) = body {
                let PositionComponent { x, y } = components.position[index]
                    .as_ref()
                    .expect("physic object doesn't have a position");

                let current_pos: glm::TVec2<f64> = glm::vec2(f64::from(*x), f64::from(*y));

                let BodyComponent {
                    net_force,
                    acceleration,
                    velocity,
                    mass,
                } = body;

                let last_acceleration = acceleration.clone();
                // TODO, test that multiplication doesn't mutate the velocity vector
                let new_pos = (*velocity * dt) + current_pos + (last_acceleration * 0.5 * dt * dt);

                let rho = 1.2;
                // this things should come from the object
                let coeff = 0.3;
                let a = 0.5;
                let air_drag = 0.5
                    * rho
                    * a
                    * coeff
                    * glm::vec2(
                        velocity.x * velocity.x * velocity.x.signum(),
                        velocity.y * velocity.y * velocity.y.signum(),
                    );

                *net_force = *net_force - air_drag;

                *acceleration = *net_force / *mass;
                let avg_acceleration = (last_acceleration + *acceleration) / 2.0;

                *velocity = *velocity + avg_acceleration * dt;

                // undo the air drag because for the next iteration it is computed from the velocity
                // *net_force = *net_force + air_drag;
                *net_force = glm::zero();

                // TODO: just store a glm::vec2 in PositionComponent?
                components.position[index] = Some(PositionComponent {
                    x: new_pos.x as f32,
                    y: new_pos.y as f32,
                });
            }
        }
    }
}

impl BodyComponent {
    pub fn new(mass: f64) -> BodyComponent {
        BodyComponent {
            net_force: glm::vec2(0.0, 0.0),
            acceleration: glm::vec2(0.0, 0.0),
            velocity: glm::vec2(0.0, 0.0),
            mass,
        }
    }

    pub fn apply_force_x(&mut self, force: f64) {
        self.net_force.x += force;
    }

    pub fn apply_force_y(&mut self, force: f64) {
        self.net_force.y += force;
    }
}
