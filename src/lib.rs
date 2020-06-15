mod arena;
mod entity_manager;
mod graphics;
pub mod systems;
pub use arena::Arena;
pub use entity_manager::*;
pub use graphics::RenderComponent;
use nalgebra_glm as glm;
use std::collections::VecDeque;
use std::convert::TryInto;
pub use systems::{
    BodyComponent, BulletComponent, CollisionComponent, HealthComponent, LogicMessage,
    OffArenaDebuffComponent,
};

const X_MAX: f32 = 800.0f32;
const Y_MAX: f32 = 800.0f32;

pub struct Game {
    systems: Systems,
    entity_manager: EntityManager,
    component_manager: ComponentManager,
    arena: Arena,
    player_movement: PlayerState,
}

struct Systems {
    render: systems::RenderSystem,
    physics: systems::PhysicsSystem,
    collision: systems::CollisionSystem,
    logic: systems::LogicSystem,
    debuff: systems::DebuffSystem,
}

impl Game {
    pub fn new() -> Game {
        let entity_manager = EntityManager::new();
        let component_manager = ComponentManager::new();
        let arena = Arena::new();
        Game {
            systems: Systems {
                render: systems::RenderSystem::new().unwrap(),
                physics: systems::PhysicsSystem::new(),
                collision: systems::CollisionSystem::new(),
                logic: systems::LogicSystem::new(),
                debuff: systems::DebuffSystem::new(),
            },
            entity_manager,
            component_manager,
            arena,
            player_movement: Default::default(),
        }
    }

    pub fn update_state(&mut self, dt: std::time::Duration) {
        let force_to_apply = 500.0;

        let player_entity = self.player_movement.id.expect("player not set");

        if let Some(direction) = &self.player_movement.rotating {
            match direction {
                RotationDirection::Left => {
                    self.component_manager.update_orientation_component(
                        player_entity,
                        |component| {
                            component.angle += std::f32::consts::PI / 800.0;
                        },
                    );
                }
                RotationDirection::Right => {
                    self.component_manager.update_orientation_component(
                        player_entity,
                        |component| {
                            component.angle -= std::f32::consts::PI / 800.0;
                        },
                    );
                }
            }
        };

        let orientation = *self
            .component_manager
            .get_orientation_component(player_entity)
            .clone()
            .unwrap();

        if let Some(direction) = &self.player_movement.moving {
            match direction {
                MovementDirection::Up => {
                    self.component_manager
                        .update_body_component(player_entity, |body| {
                            body.apply_force_x(f64::from(orientation.angle.cos()) * force_to_apply);
                            body.apply_force_y(f64::from(orientation.angle.sin()) * force_to_apply);
                        });
                }
                MovementDirection::Down => {
                    self.component_manager
                        .update_body_component(player_entity, |body| {
                            body.apply_force_x(
                                -f64::from(orientation.angle.cos()) * force_to_apply,
                            );
                            body.apply_force_y(
                                -f64::from(orientation.angle.sin()) * force_to_apply,
                            );
                        });
                }
            }
        };

        let mut logic_events = std::collections::VecDeque::new();
        if let Some(_) = self.player_movement.shooting.take() {
            if let Some(OrientationComponent { angle }) = self
                .component_manager
                .get_orientation_component(player_entity)
            {
                logic_events.push_back(LogicMessage::Shoot {
                    shooter: player_entity,
                    orientation: *angle,
                });
            }
        }

        self.systems
            .physics
            .run(dt.as_secs_f64(), &mut self.component_manager);

        self.systems
            .collision
            .run(&mut self.component_manager, |a, b| {
                // self.systems.logic.push_event(LogicMessage::Collision(a, b))
                logic_events.push_back(LogicMessage::Collision(a, b));
            });

        let entities = &mut self.entity_manager;

        self.systems.logic.run(
            &mut self.arena,
            entities,
            &mut self.component_manager,
            logic_events,
        );

        self.systems
            .debuff
            .run(&self.arena, entities, &mut self.component_manager);
    }

    pub fn render(&mut self) {
        self.systems
            .render
            .render(&self.arena, &self.component_manager);
    }

    pub fn add_player(&mut self) {
        let player_entity = self.entity_manager.next_entity();
        let player_size = 30.0;

        self.component_manager.set_position_component(
            player_entity,
            PositionComponent::new_wrapping(0.0f32, 0.0f32),
        );
        self.component_manager
            .set_render_component(player_entity, unsafe {
                RenderComponent::new_shooter(player_size, 5.0)
            });
        self.component_manager
            .set_collision_component(player_entity, CollisionComponent::new(player_size));

        self.component_manager
            .set_body_component(player_entity, BodyComponent::new(10.0, 0.4));
        self.component_manager
            .set_orientation_component(player_entity, OrientationComponent::new(0.0));
        self.component_manager
            .set_health_component(player_entity, HealthComponent::new(100));

        self.player_movement.id = Some(player_entity);
    }

    pub fn player_command(&mut self, cmd: PlayerCommand) {
        match cmd {
            PlayerCommand::Movement {
                direction,
                action: MovementAction::Start,
            } => {
                self.player_movement.moving.replace(direction);
            }
            PlayerCommand::Movement {
                direction: _,
                action: MovementAction::Stop,
            } => {
                self.player_movement.moving = None;
            }
            PlayerCommand::Rotation {
                direction,
                action: MovementAction::Start,
            } => {
                self.player_movement.rotating.replace(direction);
            }
            PlayerCommand::Rotation {
                direction: _,
                action: MovementAction::Stop,
            } => {
                self.player_movement.rotating = None;
            }
            PlayerCommand::Shoot => {
                self.player_movement.shooting = Some(());
            }
        }
    }
}

#[derive(Default)]
struct PlayerState {
    id: Option<Entity>,
    rotating: Option<RotationDirection>,
    moving: Option<MovementDirection>,
    shooting: Option<()>,
}

pub enum MovementDirection {
    Up,
    Down,
}

pub enum RotationDirection {
    Left,
    Right,
}

pub enum MovementAction {
    Start,
    Stop,
}

pub enum PlayerCommand {
    Movement {
        direction: MovementDirection,
        action: MovementAction,
    },
    Rotation {
        direction: RotationDirection,
        action: MovementAction,
    },
    Shoot,
}

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
