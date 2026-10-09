//! The loaded content database as a Bevy resource.
use bevy::prelude::*;
use u67_world::db::Db;

#[derive(Resource)]
pub struct DbRes(pub Db);
