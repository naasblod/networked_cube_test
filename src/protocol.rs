use bevy_xpbd_3d::components::*;

use lightyear::client::components::LerpFn;

use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::*;
use lightyear::utils::bevy::*;
use serde::{Deserialize, Serialize};

#[derive(Bundle)]
pub(crate) struct PlayerBundle {
    id: PlayerId,
    position: Position,
    action_state: ActionState<PlayerActions>,
    physics: PhysicsBundle,
}

impl PlayerBundle {
    pub(crate) fn new(id: ClientId, position: Vec3) -> Self {
        Self {
            id: PlayerId(id),
            position: Position(position),
            action_state: ActionState::default(),
            physics: PhysicsBundle {
                collider: Collider::cuboid(1.0, 1.0, 1.0),
                collider_density: ColliderDensity(3.0),
                rigid_body: RigidBody::Dynamic,
            },
        }
    }
}

#[derive(Bundle)]
pub(crate) struct PhysicsBundle {
    pub(crate) collider: Collider,
    pub(crate) collider_density: ColliderDensity,
    pub(crate) rigid_body: RigidBody,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerId(pub ClientId);

#[component_protocol(protocol = "MyProtocol")]
pub enum Components {
    #[protocol(sync(mode = "once"))]
    PlayerId(PlayerId),
    // #[protocol(sync(
    //     mode = "full",
    //     lerp = "PositionLinearInterpolation",
    //     corrector = "InterpolatedCorrector"
    // ))]
    // Position(Position),
    // #[protocol(sync(mode = "full", lerp = "NullInterpolator"))]
    // Position(Position),
    // #[protocol(sync(mode = "full", lerp = "NullInterpolator"))]
    // Rotation(Rotation),
    // NOTE: correction is only needed for components that are visually displayed!
    #[protocol(sync(mode = "full", lerp = "NullInterpolator"))]
    LinearVelocity(LinearVelocity),

    // #[protocol(sync(mode = "full", lerp = "NullInterpolator"))]
    // AngularVelocity(AngularVelocity),
    // #[protocol(sync(mode = "full", lerp = "NullInterpolator"))]
    // GlobalTransform(GlobalTransform),
    #[protocol(sync(mode = "full", lerp = "TransformLinearInterpolation"))]
    Transform(Transform),
}

#[derive(Channel)]
pub struct Channel1;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClientConnect {
    pub(crate) id: ClientId,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClientDisconnect {
    pub(crate) id: ClientId,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClientAssetLoadingComplete;

#[message_protocol(protocol = "MyProtocol")]
pub enum Messages {
    ClientConnect(ClientConnect),
    ClientDisconnect(ClientDisconnect),
    ClientAssetLoadingComplete(ClientAssetLoadingComplete),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect, Actionlike)]
pub enum PlayerActions {
    Up,
    Down,
    Left,
    Right,
    Jump,
}

impl LeafwingUserAction for PlayerActions {}

protocolize! {
    Self = MyProtocol,
    Message = Messages,
    Component = Components,
    Input = (),
    LeafwingInput1 = PlayerActions,
    LeafwingInput2 = NoAction2,
}

pub(crate) fn protocol() -> MyProtocol {
    let mut protocol = MyProtocol::default();
    protocol.add_channel::<Channel1>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
        ..default()
    });
    protocol
}

pub struct PositionLinearInterpolation;

impl LerpFn<Position> for PositionLinearInterpolation {
    fn lerp(start: &Position, other: &Position, t: f32) -> Position {
        let res = Position::new(start.0 * (1.0 - t) + other.0 * t);
        res
    }
}

#[derive(Component)]
pub struct LocalPlayer;
