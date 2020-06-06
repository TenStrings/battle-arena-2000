use crate::{
    BodyComponent, CollisionComponent, ComponentManager, Entity, EntityManager, RenderComponent,
};
use nalgebra_glm as glm;
use std::collections::VecDeque;

#[derive(Default)]
pub struct LogicSystem {
    msg_box: VecDeque<LogicMessage>,
}

pub enum LogicMessage {
    Collision(Entity, Entity),
    Shoot { shooter: Entity, orientation: f32 },
}

#[derive(Clone)]
pub struct BulletComponent {}

impl LogicSystem {
    pub fn new() -> LogicSystem {
        LogicSystem {
            msg_box: VecDeque::new(),
        }
    }

    pub fn run(&mut self, entity_manager: &mut EntityManager, components: &mut ComponentManager) {
        while let Some(msg) = self.msg_box.pop_back() {
            match msg {
                LogicMessage::Collision(a, b) => {
                    for e in &[a, b] {
                        if components
                            .bullet
                            .get(e.0 as usize)
                            .map(|c| c.is_some())
                            .unwrap_or(false)
                        {
                            entity_manager.remove_entity(*e);
                            components.remove_entity(*e);
                        }
                    }
                }
                LogicMessage::Shoot {
                    shooter,
                    orientation,
                } => {
                    let bullet_entity = entity_manager.next_entity();
                    let shooter_position = components.position[shooter.0 as usize]
                        .as_ref()
                        .expect("no position for shooter");

                    let shooter_hitbox = components.collision[shooter.0 as usize]
                        .as_ref()
                        .expect("no collision for shooter");

                    let bullet_size = 10.0;

                    let bullet_direction: glm::Vec2 =
                        glm::vec2(orientation.cos(), orientation.sin());

                    let bullet_position: glm::Vec2 =
                        glm::vec2(shooter_position.x, shooter_position.y)
                            + (bullet_size + shooter_hitbox.radius) * bullet_direction;

                    components.set_position_component(bullet_entity, bullet_position.into());

                    components.set_render_component(bullet_entity, unsafe {
                        RenderComponent::new_circle(bullet_size)
                    });

                    components.set_collision_component(
                        bullet_entity,
                        CollisionComponent::new(bullet_size),
                    );

                    let mut body = BodyComponent::new(10.0, 0.1);
                    let bullet_speed = 1000.0;
                    body.velocity =
                        glm::DVec2::new(bullet_direction.x.into(), bullet_direction.y.into())
                            * bullet_speed;

                    components.set_body_component(bullet_entity, body);
                    components.set_bullet_component(bullet_entity, BulletComponent::default());
                }
            }
        }
    }

    pub fn push_event(&mut self, event: LogicMessage) {
        self.msg_box.push_back(event);
    }
}

impl Default for BulletComponent {
    fn default() -> BulletComponent {
        BulletComponent {}
    }
}
