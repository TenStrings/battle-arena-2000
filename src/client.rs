use crate::network::{ClientMessage, UpdateMessage};
use crate::systems;
use crate::{Arena, ComponentManager, Entity, EntityManager};
use crate::{
    BodyComponent, BulletComponent, CollisionComponent, HealthComponent, LogicMessage,
    OffArenaDebuffComponent, OrientationComponent, PositionComponent, RenderComponent,
};
use crate::{MovementAction, MovementDirection, PlayerCommand, PlayerState, RotationDirection};
use log::{debug, error, info, trace};
use std::net::{TcpListener, TcpStream};

pub struct Client {
    systems: Systems,
    entity_manager: EntityManager,
    component_manager: ComponentManager,
    arena: Arena,
    server: TcpStream,
    player_state: PlayerState,
}

struct Systems {
    render: systems::RenderSystem,
}

impl Client {
    pub fn new(server: TcpStream) -> Client {
        let entity_manager = EntityManager::new();
        let component_manager = ComponentManager::new();
        let arena = Arena::new();

        server
            .set_read_timeout(Some(std::time::Duration::from_millis(10)))
            .unwrap();

        Client {
            systems: Systems {
                render: systems::RenderSystem::new().expect("couldn't initialize render system"),
            },
            entity_manager,
            component_manager,
            arena,
            server,
            player_state: PlayerState::default(),
        }
    }

    pub fn run(&mut self) {
        trace!("client run");
        let msg: Result<UpdateMessage, _> = bincode::deserialize_from(&self.server);
        match msg {
            Ok(msg) => {
                debug!("running client update with msg: {:?}", &msg);
                use UpdateMessage::*;
                match msg {
                    RotateEntity(entity, angle) => {
                        self.component_manager
                            .update_orientation_component(entity, |orientation| {
                                orientation.angle += angle
                            });
                    }
                    Shoot(shooter, angle) => unimplemented!(),
                    SetPosition(entity, x, y) => {
                        self.component_manager.update_position_component(
                            entity,
                            |position_component| {
                                position_component.x = x;
                                position_component.y = y;
                            },
                        );
                    }
                    Joined(u32) => self.add_player(),
                    AddPlayer => unimplemented!(),
                }
            }
            Err(err) => {
                // error!(target: "client", "failed reading message {:?}", err);
            }
        }
    }

    pub fn render(&mut self) {
        trace!(target: "client", "running render");
        self.systems
            .render
            .render(&self.arena, &self.component_manager);
    }

    pub fn player_command(&mut self, cmd: PlayerCommand) {
        match cmd {
            PlayerCommand::Movement {
                direction,
                action: MovementAction::Start,
            } => {
                self.player_state.moving.replace(direction);
            }
            PlayerCommand::Movement {
                direction: _,
                action: MovementAction::Stop,
            } => {
                self.player_state.moving = None;
            }
            PlayerCommand::Rotation {
                direction,
                action: MovementAction::Start,
            } => {
                self.player_state.rotating.replace(direction);
            }
            PlayerCommand::Rotation {
                direction: _,
                action: MovementAction::Stop,
            } => {
                self.player_state.rotating = None;
            }
            PlayerCommand::Shoot => {
                self.player_state.shooting = Some(());
            }
        }

        debug!(target: "client", "sending player command: {:?}", &self.player_state);
        if let Err(_) = bincode::serialize_into(&self.server, &self.player_state) {
            error!(target: "client", "couldn't send player command to server");
        }
    }

    pub fn add_player(&mut self) {
        let player_entity = self.entity_manager.next_entity();
        let player_size = 30.0;

        self.component_manager.set_position_component(
            player_entity,
            PositionComponent::new_wrapping(0.0f32, 0.0f32),
        );

        self.component_manager
            .set_orientation_component(player_entity, OrientationComponent::new(0.0));

        self.component_manager
            .set_render_component(player_entity, unsafe {
                RenderComponent::new_shooter(player_size, 5.0)
            });
    }
}
