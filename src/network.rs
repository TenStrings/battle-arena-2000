use crate::Entity;
use crate::PlayerState;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum UpdateMessage {
    RotateEntity(Entity, f32),
    Shoot(Entity, f32),
    SetPosition(Entity, f32, f32),
    Joined(u32),
    AddPlayer,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    PlayerCommand(PlayerState),
}
