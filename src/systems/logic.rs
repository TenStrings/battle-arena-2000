use crate::{
    Arena, BodyComponent, CollisionComponent, ComponentManager, Entity, EntityManager,
    OffArenaDebuffComponent, RenderComponent,
};
use nalgebra_glm as glm;
use std::collections::VecDeque;

pub struct LogicSystem {
    msg_box: VecDeque<LogicMessage>,
    timers: Vec<Timer>,
    last_instant: std::time::Instant,
}

pub enum LogicMessage {
    Collision(Entity, Entity),
    Shoot { shooter: Entity, orientation: f32 },
    ShrinkMap(f32),
}

#[derive(Clone)]
pub struct BulletComponent {}

#[derive(Clone)]
pub struct HealthComponent(pub u32);

struct Timer {
    remaining: std::time::Duration,
    on_expiration: Box<
        dyn FnMut(
            &mut VecDeque<LogicMessage>,
            &mut EntityManager,
            &mut ComponentManager,
        ) -> Option<std::time::Duration>,
    >,
}

impl LogicSystem {
    pub fn new() -> LogicSystem {
        let map_shrink_timer = Timer {
            remaining: std::time::Duration::from_secs(5),
            on_expiration: Box::new(|logic_msg_box, _entity_manager, _component_manager| {
                logic_msg_box.push_back(LogicMessage::ShrinkMap(0.1));
                Some(std::time::Duration::from_secs(5))
            }),
        };
        LogicSystem {
            msg_box: VecDeque::new(),
            timers: vec![map_shrink_timer],
            last_instant: std::time::Instant::now(),
        }
    }

    pub fn run(
        &mut self,
        arena: &mut Arena,
        entity_manager: &mut EntityManager,
        components: &mut ComponentManager,
    ) {
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
                LogicMessage::ShrinkMap(amount) => {
                    arena.shrink(amount);
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

        let mut entities_to_delete = vec![];

        for entity in entity_manager.iter() {
            let body = components.get_body_component(entity);
            let bullet = components.get_bullet_component(entity);
            let position = components.get_position_component(entity);

            if bullet.is_some() {
                if glm::magnitude(&body.unwrap().velocity) < 60.0 {
                    entities_to_delete.push(entity);
                }
            }

            if let Some(health) = components.get_health_component(entity) {
                if health.0 == 0u32 {
                    entities_to_delete.push(entity);
                }
            }

            if !position.map(|pos| arena.contains(pos)).unwrap_or(false) {
                let needs_debuff = components.get_off_arena_debuff_component(entity).is_none();

                if needs_debuff {
                    components
                        .set_off_arena_debuff_component(entity, OffArenaDebuffComponent::default());
                }
            }
        }

        let new_instant = std::time::Instant::now();
        let dt = new_instant.duration_since(self.last_instant);
        self.last_instant = new_instant;

        let mut timers_to_delete = vec![];

        for (index, timer) in self.timers.iter_mut().enumerate() {
            if let Some(time_remaining) = timer.remaining.checked_sub(dt) {
                timer.remaining = time_remaining;
            } else {
                if let Some(new_duration) =
                    (timer.on_expiration)(&mut self.msg_box, entity_manager, components)
                {
                    timer.remaining = new_duration;
                } else {
                    timers_to_delete.push(index);
                }
            }
        }

        for index in timers_to_delete {
            self.timers.swap_remove(index);
        }

        for entity in entities_to_delete {
            entity_manager.remove_entity(entity);
            components.remove_entity(entity);
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

impl HealthComponent {
    pub fn new(health: u32) -> HealthComponent {
        HealthComponent(health)
    }
}
