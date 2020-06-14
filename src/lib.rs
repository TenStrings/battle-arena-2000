mod arena;
mod entity_manager;
mod graphics;
pub mod systems;
pub use arena::Arena;
pub use entity_manager::*;
pub use graphics::RenderComponent;
use nalgebra_glm as glm;
use std::convert::TryInto;
pub use systems::{
    BodyComponent, BulletComponent, CollisionComponent, HealthComponent, LogicMessage,
    OffArenaDebuffComponent,
};

const X_MAX: f32 = 800.0f32;
const Y_MAX: f32 = 800.0f32;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Entity(u32);

#[derive(Debug, Clone, Copy)]
pub struct PositionComponent {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct OrientationComponent {
    pub angle: f32,
}

// TODO: make things private?
#[derive(Default)]
pub struct ComponentManager {
    position: Vec<Option<PositionComponent>>,
    render: Vec<Option<RenderComponent>>,
    body: Vec<Option<BodyComponent>>,
    collision: Vec<Option<CollisionComponent>>,
    bullet: Vec<Option<BulletComponent>>,
    orientation: Vec<Option<OrientationComponent>>,
    health: Vec<Option<HealthComponent>>,
    off_arena: Vec<Option<OffArenaDebuffComponent>>,
}

impl ComponentManager {
    pub fn new() -> Self {
        ComponentManager {
            position: vec![],
            render: vec![],
            body: vec![],
            collision: vec![],
            bullet: vec![],
            orientation: vec![],
            health: vec![],
            off_arena: vec![],
        }
    }

    fn get_component<T>(pool: &[Option<T>], entity: Entity) -> Option<&T> {
        let index: usize = entity.0.try_into().unwrap();
        if let Some(entry) = pool.get(index) {
            entry.as_ref()
        } else {
            None
        }
    }

    fn set_component<T: Clone>(pool: &mut Vec<Option<T>>, entity: Entity, component: T) {
        let index: usize = entity.0.try_into().unwrap();
        if let Some(entry) = pool.get_mut(index) {
            entry.replace(component);
        } else {
            pool.resize(index + 1, None);
            Self::set_component(pool, entity, component);
        }
    }

    pub fn set_render_component(&mut self, entity: Entity, component: RenderComponent) {
        Self::set_component(&mut self.render, entity, component);
    }

    pub fn set_position_component(&mut self, entity: Entity, component: PositionComponent) {
        Self::set_component(&mut self.position, entity, component);
    }

    pub fn set_body_component(&mut self, entity: Entity, component: BodyComponent) {
        Self::set_component(&mut self.body, entity, component);
    }

    pub fn set_collision_component(&mut self, entity: Entity, component: CollisionComponent) {
        Self::set_component(&mut self.collision, entity, component);
    }

    pub fn set_bullet_component(&mut self, entity: Entity, component: BulletComponent) {
        Self::set_component(&mut self.bullet, entity, component);
    }

    pub fn set_orientation_component(&mut self, entity: Entity, component: OrientationComponent) {
        Self::set_component(&mut self.orientation, entity, component);
    }

    pub fn set_health_component(&mut self, entity: Entity, component: HealthComponent) {
        Self::set_component(&mut self.health, entity, component);
    }

    pub fn set_off_arena_debuff_component(
        &mut self,
        entity: Entity,
        component: OffArenaDebuffComponent,
    ) {
        Self::set_component(&mut self.off_arena, entity, component);
    }

    pub fn get_position_component(&self, entity: Entity) -> Option<&PositionComponent> {
        Self::get_component(&self.position, entity)
    }

    pub fn get_orientation_component(&self, entity: Entity) -> Option<&OrientationComponent> {
        Self::get_component(&self.orientation, entity)
    }

    pub fn get_body_component(&self, entity: Entity) -> Option<&BodyComponent> {
        Self::get_component(&self.body, entity)
    }

    pub fn get_bullet_component(&self, entity: Entity) -> Option<&BulletComponent> {
        Self::get_component(&self.bullet, entity)
    }

    pub fn get_health_component(&self, entity: Entity) -> Option<&HealthComponent> {
        Self::get_component(&self.health, entity)
    }

    pub fn get_off_arena_debuff_component(
        &self,
        entity: Entity,
    ) -> Option<&OffArenaDebuffComponent> {
        Self::get_component(&self.off_arena, entity)
    }

    pub fn update_position_component(
        &mut self,
        entity: Entity,
        mut f: impl FnMut(&mut PositionComponent),
    ) {
        let index: usize = entity.0.try_into().unwrap();
        if let Some(entry) = self.position.get_mut(index) {
            if let Some(entry) = entry {
                f(entry)
            }
        }
    }

    pub fn update_body_component(&mut self, entity: Entity, mut f: impl FnMut(&mut BodyComponent)) {
        let index: usize = entity.0.try_into().unwrap();
        if let Some(entry) = self.body.get_mut(index) {
            if let Some(entry) = entry {
                f(entry)
            }
        }
    }

    pub fn update_orientation_component(
        &mut self,
        entity: Entity,
        mut f: impl FnMut(&mut OrientationComponent),
    ) {
        let index: usize = entity.0.try_into().unwrap();
        if let Some(entry) = self.orientation.get_mut(index) {
            if let Some(entry) = entry {
                f(entry)
            }
        }
    }

    pub fn update_health_component(
        &mut self,
        entity: Entity,
        mut f: impl FnMut(&mut HealthComponent),
    ) {
        let index: usize = entity.0.try_into().unwrap();
        if let Some(entry) = self.health.get_mut(index) {
            if let Some(entry) = entry {
                f(entry)
            }
        }
    }

    pub fn update_off_arena_debuff_component(
        &mut self,
        entity: Entity,
        mut f: impl FnMut(&mut OffArenaDebuffComponent),
    ) {
        let index: usize = entity.0.try_into().unwrap();
        if let Some(entry) = self.off_arena.get_mut(index) {
            if let Some(entry) = entry {
                f(entry)
            }
        }
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        if let Some(ref mut c) = self.position.get_mut(entity.0 as usize) {
            **c = None;
        }
        if let Some(ref mut c) = self.render.get_mut(entity.0 as usize) {
            **c = None;
        }
        if let Some(ref mut c) = self.body.get_mut(entity.0 as usize) {
            **c = None;
        }
        if let Some(ref mut c) = self.collision.get_mut(entity.0 as usize) {
            **c = None;
        }
        if let Some(ref mut c) = self.bullet.get_mut(entity.0 as usize) {
            **c = None;
        }
        if let Some(ref mut c) = self.health.get_mut(entity.0 as usize) {
            **c = None;
        }
        if let Some(ref mut c) = self.off_arena.get_mut(entity.0 as usize) {
            **c = None;
        }
    }
}

impl PositionComponent {
    pub fn new_wrapping(x: f32, y: f32) -> PositionComponent {
        let x = if x < 0.0 { x + std::f32::MAX } else { x };
        let y = if y < 0.0 { y + std::f32::MAX } else { y };
        PositionComponent {
            x: x % X_MAX,
            y: y % Y_MAX,
        }
    }

    pub fn set_x_wrap(&mut self, x: f32) {
        self.x = x % X_MAX;
    }

    pub fn set_y_wrap(&mut self, y: f32) {
        self.y = y % Y_MAX;
    }
}

impl OrientationComponent {
    pub fn new(angle: f32) -> OrientationComponent {
        OrientationComponent { angle }
    }
}

impl From<glm::Vec2> for PositionComponent {
    fn from(pos: glm::Vec2) -> PositionComponent {
        PositionComponent { x: pos.x, y: pos.y }
    }
}

impl Into<glm::Vec2> for PositionComponent {
    fn into(self) -> glm::Vec2 {
        glm::vec2(self.x, self.y)
    }
}
