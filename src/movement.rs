use bevy::prelude::*;
use bevy_tnua::{
    builtins::{TnuaBuiltinJump, TnuaBuiltinWalk},
    control_helpers::TnuaCrouchEnforcerPlugin,
    controller::{TnuaController, TnuaControllerPlugin},
    TnuaUserControlsSystemSet,
};
use bevy_tnua_xpbd3d::TnuaXpbd3dPlugin;
use bevy_xpbd_3d::{
    components::{Position, Rotation},
    plugins::{setup::Physics, PhysicsPlugins},
};
use leafwing_input_manager::action_state::ActionState;
use lightyear::{prelude::client::*, shared::tick_manager::TickManager};

use crate::protocol::{LocalPlayer, MyProtocol, PlayerActions};

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LeafwingInputPlugin::<MyProtocol, PlayerActions>::new(
            LeafwingInputConfig::<PlayerActions> {
                send_diffs_only: true,
                ..default()
            },
        ))
        .add_systems(Update, (draw_interpolated_boxes, draw_confirmed_boxes));
    }
}

fn player_movement(
    tick_manager: Res<TickManager>,
    mut action_query: Query<
        (
            Entity,
            &Position,
            &mut TnuaController,
            &ActionState<PlayerActions>,
        ),
        With<LocalPlayer>,
    >,
) {
    for (entity, position, mut controller, action_state) in action_query.iter_mut() {
        info!(?entity, tick = ?tick_manager.tick(), ?position, actions = ?action_state.get_pressed(), "applying movement to player");

        shared_movement_behaviour(&mut controller, action_state);
    }
}

pub fn shared_movement_behaviour(
    controller: &mut TnuaController,
    action: &ActionState<PlayerActions>,
) {
    let mut direction = Vec3::ZERO;

    if action.pressed(&PlayerActions::Up) {
        direction -= Vec3::Z;
    }

    if action.pressed(&PlayerActions::Down) {
        direction += Vec3::Z;
    }

    if action.pressed(&PlayerActions::Left) {
        direction -= Vec3::X;
    }

    if action.pressed(&PlayerActions::Right) {
        direction += Vec3::X;
    }

    controller.basis(TnuaBuiltinWalk {
        desired_velocity: direction.normalize_or_zero() * 0.33,
        float_height: 2.0,
        ..Default::default()
    });

    if action.pressed(&PlayerActions::Jump) {
        controller.action(TnuaBuiltinJump {
            height: 4.0,
            ..Default::default()
        });
    }
}

fn draw_interpolated_boxes(
    mut gizmos: Gizmos,
    players: Query<(&Position, &Rotation), Or<(With<Interpolated>, With<LocalPlayer>)>>,
) {
    for (position, rotation) in &players {
        gizmos.cuboid(
            Transform::from_xyz(position.x, position.y, position.z)
                .with_rotation(rotation.0)
                .with_scale(Vec3::splat(1.0)),
            Color::BLUE,
        );
    }
}

fn draw_confirmed_boxes(
    mut gizmos: Gizmos,
    players: Query<(&Position, &Rotation), With<Confirmed>>,
) {
    for (position, rotation) in &players {
        gizmos.cuboid(
            Transform::from_xyz(position.x, position.y, position.z)
                .with_rotation(rotation.0)
                .with_scale(Vec3::splat(1.0)),
            Color::RED,
        );
    }
}
