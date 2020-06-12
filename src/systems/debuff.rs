use crate::{Arena, ComponentManager, EntityManager};

#[derive(Clone, Debug)]
pub struct OffArenaDebuffComponent {
    remaining: std::time::Duration,
}

pub struct DebuffSystem {
    last_instant: std::time::Instant,
}

impl DebuffSystem {
    pub fn new() -> Self {
        Self {
            last_instant: std::time::Instant::now(),
        }
    }

    pub fn run(
        &mut self,
        arena: &Arena,
        entity_manager: &EntityManager,
        components: &mut ComponentManager,
    ) {
        let new_instant = std::time::Instant::now();
        let dt = new_instant.duration_since(self.last_instant);
        self.last_instant = new_instant;

        for entity in entity_manager.iter() {
            let off_arena = components.get_off_arena_debuff_component(entity);

            let mut new = if let Some(timer) = off_arena {
                if let Some(remaining) = timer.remaining.checked_sub(dt) {
                    Some(OffArenaDebuffComponent { remaining })
                } else {
                    let position = components.get_position_component(entity);

                    if !position.map(|pos| arena.contains(pos)).unwrap_or(false) {
                        components.update_health_component(entity, |health| {
                            health.0 = health.0.saturating_sub(10);
                        });
                        Some(OffArenaDebuffComponent::default())
                    } else {
                        None
                    }
                }
            } else {
                None
            };

            if let Some(debuff) = new.take() {
                components.update_off_arena_debuff_component(entity, |component| {
                    *component = debuff.clone()
                });
            }
        }
    }
}

impl OffArenaDebuffComponent {
    pub fn new() -> OffArenaDebuffComponent {
        OffArenaDebuffComponent {
            remaining: std::time::Duration::from_millis(500),
        }
    }
}

impl Default for OffArenaDebuffComponent {
    fn default() -> OffArenaDebuffComponent {
        OffArenaDebuffComponent::new()
    }
}
