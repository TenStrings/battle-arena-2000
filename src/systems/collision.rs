use crate::ComponentManager;
use crate::PositionComponent;
use nalgebra_glm as glm;

pub struct CollisionSystem {}

#[derive(Clone, Debug)]
pub struct CollisionComponent {
    radius: f32,
}

impl CollisionSystem {
    pub fn new() -> CollisionSystem {
        CollisionSystem {}
    }

    pub fn run(&self, components: &mut ComponentManager) {
        for (index, collision) in components.collision.iter().enumerate() {
            if let Some(collision) = collision {
                let PositionComponent { x, y } = components.position[index]
                    .as_ref()
                    .expect("collision object doesn't have a position");

                let current_pos: glm::TVec2<f64> = glm::vec2(f64::from(*x), f64::from(*y));

                for (index_other, collision_other) in components.collision.iter().enumerate() {
                    match collision_other {
                        Some(collision_other) if index != index_other => {
                            let PositionComponent { x, y } = components.position[index_other]
                                .as_ref()
                                .expect("collision object doesn't have a position");

                            let other_pos: glm::TVec2<f64> =
                                glm::vec2(f64::from(*x), f64::from(*y));

                            let distance = glm::distance(&current_pos, &other_pos);

                            if distance < (collision.radius + collision_other.radius).into() {
                                println!("collision detected");
                            }
                        }
                        _ => continue,
                    }
                }
            }
        }
    }
}

impl CollisionComponent {
    pub fn new(radius: f32) -> CollisionComponent {
        CollisionComponent { radius }
    }
}
