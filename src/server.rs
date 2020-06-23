use crate::network::{ClientMessage, UpdateMessage};
use crate::systems;
use crate::{Arena, ComponentManager, Entity, EntityManager};
use crate::{
    BodyComponent, BulletComponent, CollisionComponent, HealthComponent, LogicMessage,
    OffArenaDebuffComponent, OrientationComponent, PositionComponent,
};
use crate::{MovementAction, MovementDirection, PlayerCommand, PlayerState, RotationDirection};
use log::{debug, error, info, trace};
use std::collections::VecDeque;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};

pub struct Server {
    systems: Systems,
    entity_manager: EntityManager,
    component_manager: ComponentManager,
    arena: Arena,
    player_movement: PlayerState,
    clients: Vec<TcpStream>,
}

struct Systems {
    physics: systems::PhysicsSystem,
    collision: systems::CollisionSystem,
    logic: systems::LogicSystem,
    debuff: systems::DebuffSystem,
}

impl Server {
    pub fn new() -> Server {
        let entity_manager = EntityManager::new();
        let component_manager = ComponentManager::new();
        let arena = Arena::new();
        Server {
            systems: Systems {
                physics: systems::PhysicsSystem::new(),
                collision: systems::CollisionSystem::new(),
                logic: systems::LogicSystem::new(),
                debuff: systems::DebuffSystem::new(),
            },
            entity_manager,
            component_manager,
            arena,
            player_movement: Default::default(),
            clients: Vec::default(),
        }
    }

    pub fn add_client(&mut self, stream: TcpStream) {
        info!(target: "server", "adding new client");
        stream
            .set_read_timeout(Some(std::time::Duration::from_millis(10)))
            .unwrap();

        self.add_player();

        let player_id = 0;
        bincode::serialize_into(&stream, &UpdateMessage::Joined(player_id)).unwrap();

        self.clients.push(stream);
    }

    pub fn process_client_messages(&mut self) {
        for client in &self.clients {
            let msg: Result<ClientMessage, _> = bincode::deserialize_from(client);
            match msg {
                Ok(ClientMessage::PlayerCommand(cmd)) => {
                    debug!("received player command {:?}", &cmd);
                    self.player_movement = cmd;
                }
                Err(err) => {
                    // error!(target: "server", "error reading client message {:?}", err);
                }
            }
        }
    }

    fn update_player_state(
        &mut self,
        updates: &mut Vec<UpdateMessage>,
        logic_events: &mut VecDeque<LogicMessage>,
    ) {
        let player_entity = if let Some(player_entity) = self.player_movement.id {
            player_entity
        } else {
            return;
        };

        let force_to_apply = 500.0;
        let rotation_angle = std::f32::consts::PI / 800.0;

        if let Some(direction) = &self.player_movement.rotating {
            debug!("player rotating {:?}", &direction);
            let angle = match direction {
                RotationDirection::Left => rotation_angle,
                RotationDirection::Right => -rotation_angle,
            };

            self.component_manager
                .update_orientation_component(player_entity, |component| {
                    component.angle += angle;
                });

            updates.push(UpdateMessage::RotateEntity(player_entity, -rotation_angle));
        };

        let orientation = *self
            .component_manager
            .get_orientation_component(player_entity)
            .clone()
            .unwrap();

        if let Some(direction) = &self.player_movement.moving {
            debug!("player moving {:?}", &direction);
            let (x, y) = match direction {
                MovementDirection::Up => (
                    f64::from(orientation.angle.cos()) * force_to_apply,
                    f64::from(orientation.angle.sin()) * force_to_apply,
                ),
                MovementDirection::Down => (
                    -f64::from(orientation.angle.cos()) * force_to_apply,
                    -f64::from(orientation.angle.sin()) * force_to_apply,
                ),
            };

            self.component_manager
                .update_body_component(player_entity, |body| {
                    debug!("applying force {} {}", &x, &y);
                    body.apply_force_x(x);
                    body.apply_force_y(y);
                });
        };

        if self.player_movement.shooting.take().is_some() {
            if let Some(OrientationComponent { angle }) = self
                .component_manager
                .get_orientation_component(player_entity)
            {
                logic_events.push_back(LogicMessage::Shoot {
                    shooter: player_entity,
                    orientation: *angle,
                });

                updates.push(UpdateMessage::Shoot(player_entity, *angle));
            }
        }
    }

    pub fn update_state(&mut self, dt: std::time::Duration) -> Vec<UpdateMessage> {
        trace!("running update state with delta {:?}", dt);
        let mut updates = vec![];
        let mut logic_events = std::collections::VecDeque::new();

        self.update_player_state(&mut updates, &mut logic_events);

        self.systems
            .collision
            .run(&mut self.component_manager, |a, b| {
                logic_events.push_back(LogicMessage::Collision(a, b));
            });

        self.systems
            .physics
            .run(dt.as_secs_f64(), &mut self.component_manager);

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

        for entity in self.entity_manager.iter() {
            if let Some(pos) = self.component_manager.get_position_component(entity) {
                updates.push(UpdateMessage::SetPosition(entity, pos.x, pos.y))
            }
        }

        for client in &mut self.clients {
            for update in &updates {
                trace!("writing update to client {:?}", update);
                if let Err(err) = client.write(&bincode::serialize(&update).unwrap()) {
                    error!("couldn't send message to client {}", err);
                }
            }
        }

        updates
    }

    fn add_player(&mut self) {
        let player_entity = self.entity_manager.next_entity();
        let player_size = 30.0;

        self.component_manager.set_position_component(
            player_entity,
            PositionComponent::new_wrapping(0.0f32, 0.0f32),
        );

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
}
