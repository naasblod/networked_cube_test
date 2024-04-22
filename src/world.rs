use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};
use bevy_xpbd_3d::{components::RigidBody, plugins::collision::Collider};

use crate::client::GameClientState;

pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut next_state: ResMut<NextState<GameClientState>>,
) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            color: Color::hex("fffffb").unwrap(),
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 20.0, 0.0),
            rotation: Quat::from_xyzw(-0.5, 0.0, 0.0, 1.0),
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder { ..default() }.into(),
        ..default()
    });
    // .insert(Name::new("Sun"))
    // .insert(Sun);

    commands.insert_resource(AmbientLight {
        color: Color::hex("fffffb").unwrap(),
        brightness: 0.1, // lower at night tho
    });

    // Spawn the ground.
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(128.0, 128.0)),
            material: materials.add(Color::WHITE),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        },
        RigidBody::Static,
        Collider::halfspace(Vec3::Y),
    ));

    // Loaded now try to connect
    next_state.set(GameClientState::Connecting);
}
