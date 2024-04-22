use bevy::log::{debug, info};
use bevy::utils::Duration;
use lightyear::shared::{
    config::{Mode, SharedConfig},
    tick_manager::TickConfig,
};
use bevy_xpbd_3d::prelude::*;
use bevy::prelude::*;
use bevy_tnua::control_helpers::TnuaCrouchEnforcerPlugin;
use bevy_tnua::controller::{TnuaController, TnuaControllerPlugin};
use bevy_tnua::TnuaUserControlsSystemSet;
use bevy_tnua_xpbd3d::TnuaXpbd3dPlugin;
use leafwing_input_manager::action_state::ActionState;
use lightyear::prelude::*;
use lightyear::prelude::client::{Confirmed, Rollback, RollbackState};
use crate::movement::shared_movement_behaviour;
use crate::protocol::*;

pub const FIXED_TIMESTEP_HZ: f64 = 64.0;

pub fn shared_config(mode: Mode) -> SharedConfig {
    SharedConfig {
        client_send_interval: Duration::default(),
        server_send_interval: Duration::from_millis(40),
        tick: TickConfig {
            tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
        },
        mode,
    }
}

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::new(FixedUpdate));
        app.insert_resource(Time::new_with(Physics::fixed_once_hz(FIXED_TIMESTEP_HZ)));
        app.add_plugins(TnuaXpbd3dPlugin::new(FixedUpdate));
        app.add_plugins(TnuaControllerPlugin::new(FixedUpdate));
        app.add_plugins(TnuaCrouchEnforcerPlugin::new(FixedUpdate));
        app.add_systems(FixedUpdate, movement.in_set(TnuaUserControlsSystemSet));

        app.add_systems(FixedPostUpdate, after_physics_log);
        app.add_systems(Last, last_log);
    }
}

pub(crate) fn after_physics_log(
    tick_manager: Res<TickManager>,
    rollback: Option<Res<Rollback>>,
    players: Query<
        (Entity, &Position),
    >,
) {
    let mut tick = tick_manager.tick();
    if let Some(rollback) = rollback {
        if let RollbackState::ShouldRollback { current_tick } = rollback.state {
            tick = current_tick;
        }
    }
    for (entity, position) in players.iter() {
        info!(
            ?tick,
            ?entity,
            ?position,
            "Player after physics update"
        );
    }
}

pub(crate) fn last_log(
    tick_manager: Res<TickManager>,
    players: Query<
        (
            Entity,
            &Position,
        ),
        (Without<Confirmed>, With<PlayerId>),
    >,
) {
    let tick = tick_manager.tick();
    for (entity, position) in players.iter() {
        info!(?tick, ?entity, ?position, "Player LAST update");
        info!(
            ?tick,
            ?entity,
            "Player LAST update"
        );
    }
}

fn movement(
    tick_manager: Res<TickManager>,
    mut action_query: Query<
        (
            Entity,
            &Position,
            &mut TnuaController,
            &ActionState<PlayerActions>,
        ),
        Or<(With<LocalPlayer>, With<Replicate>)>,
    >,
) {
    for (entity, position, mut controller, action_state) in action_query.iter_mut() {
        info!(?entity, tick = ?tick_manager.tick(), ?position, actions = ?action_state.get_pressed(), "applying movement to player");

        shared_movement_behaviour(&mut controller, action_state);
    }
}