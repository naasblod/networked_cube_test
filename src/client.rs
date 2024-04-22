use bevy::render::view::ColorGrading;
use bevy::window::PresentMode;
use bevy::{core_pipeline::tonemapping::Tonemapping, prelude::*};
use bevy_tnua::controller::TnuaControllerBundle;
use bevy_tnua_xpbd3d::TnuaXpbd3dSensorShape;
use bevy_xpbd_3d::{
    components::{ColliderDensity, LockedAxes, RigidBody},
    plugins::collision::Collider,
};
use leafwing_input_manager::{action_state::ActionState, input_map::InputMap, InputManagerBundle};

use std::net::{Ipv4Addr, SocketAddr};

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use lightyear::shared::config::Mode;

use bevy::log::{Level, LogPlugin};

use crate::{
    movement::MovementPlugin,
    protocol::{
        protocol, Channel1, ClientAssetLoadingComplete, ClientConnectionManager, LocalPlayer,
        PlayerActions, PlayerId,
    },
    shared::shared_config,
    world::setup_world,
};

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameClientState {
    #[default]
    Loading,
    Connecting,
    Playing,
}

pub fn client_app(net_config: client::NetConfig) -> App {
    let mut app = App::new();

    let client_config = client::ClientConfig {
        shared: shared_config(Mode::Separate),
        net: net_config,
        ..default()
    };

    let plugin_config = client::PluginConfig::new(client_config, protocol());

    app.init_state::<GameClientState>();

    app.add_plugins(lightyear::prelude::client::ClientPlugin::new(plugin_config));

    app.insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)));

    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Cubes".to_string(),
                    resolution: (1920., 1080.).into(),
                    present_mode: PresentMode::AutoVsync,
                    position: WindowPosition::Centered(MonitorSelection::Primary),
                    ..default()
                }),
                ..default()
            })
            .set(LogPlugin {
                update_subscriber: None,
                level: Level::INFO,
                filter: "wgpu=error,symphonia_core=error,symphonia_format_ogg=error".to_string(),
            }),
        // WorldPlugin,
        WorldInspectorPlugin::new(),
        MovementPlugin,
    ));

    app.add_systems(OnEnter(GameClientState::Loading), setup_world);
    app.add_systems(OnEnter(GameClientState::Connecting), try_connecting);
    app.add_systems(
        PreUpdate,
        handle_connection
            .after(MainSet::Receive)
            .run_if(in_state(GameClientState::Connecting)),
    );

    app.add_systems(
        Update,
        wait_for_local_player_spawn.run_if(in_state(GameClientState::Connecting)),
    );

    app
}

fn try_connecting(mut next_state: ResMut<NextState<NetworkingState>>) {
    next_state.set(NetworkingState::Connecting);
}

fn handle_connection(
    mut client: ResMut<ClientConnectionManager>,
    mut connection_event: EventReader<ConnectEvent>,
) {
    for _event in connection_event.read() {
        // let server know we've finished loading assets
        client
            .send_message_to_target::<Channel1, ClientAssetLoadingComplete>(
                ClientAssetLoadingComplete {},
                NetworkTarget::None,
            )
            .unwrap_or_else(|e| {
                error!("Failed to send message: {:?}", e);
            });
    }
}

fn wait_for_local_player_spawn(
    confirmed: Query<(Entity, &PlayerId), Added<Predicted>>,
    client_config: Res<ClientConfig>,
    mut next_state: ResMut<NextState<GameClientState>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, player_id) in confirmed.iter() {
        info!("Adding player: {:?}", player_id);

        let local_client_id = ClientId::Netcode(get_local_client_id(&client_config.net).unwrap());

        if player_id.0 == local_client_id {
            info!("Player to add is local");

            commands
                .entity(entity)
                .insert(LocalPlayer)
                .insert(PbrBundle {
                    mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                    material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
                    transform: Transform::from_xyz(0.0, 0.5, 0.0),
                    ..default()
                })
                .insert(InputManagerBundle::<PlayerActions> {
                    action_state: ActionState::default(),
                    input_map: InputMap::new([
                        (PlayerActions::Up, KeyCode::KeyW),
                        (PlayerActions::Down, KeyCode::KeyS),
                        (PlayerActions::Left, KeyCode::KeyA),
                        (PlayerActions::Right, KeyCode::KeyD),
                        (PlayerActions::Jump, KeyCode::Space),
                    ]),
                })
                .insert((
                    Collider::cuboid(1.0, 1.0, 1.0),
                    ColliderDensity(3.0),
                    RigidBody::Dynamic,
                    LockedAxes::ROTATION_LOCKED,
                    TnuaControllerBundle::default(),
                    TnuaXpbd3dSensorShape(Collider::cuboid(0.98, 0.0, 0.98)),
                ));

            // TODO move this somewhere else ?
            commands.spawn((
                Camera3dBundle {
                    camera: Camera {
                        hdr: true,
                        ..default()
                    },
                    tonemapping: Tonemapping::ReinhardLuminance,
                    color_grading: ColorGrading {
                        exposure: 0.0,
                        gamma: 1.07,
                        pre_saturation: 1.0,
                        post_saturation: 1.1,
                        ..default()
                    },
                    projection: PerspectiveProjection {
                        fov: (75.0_f32).to_radians(),
                        // near: 0.11,
                        // far: 1000.0,
                        aspect_ratio: 1920.0 / 1080.0,
                        ..default()
                    }
                    .into(),
                    transform: Transform::from_xyz(0.0, 4.8, 5.0)
                        .with_rotation(Quat::from_xyzw(-0.5, 0.0, 0.0, 1.0)),
                    ..default()
                },
                FogSettings {
                    color: Color::rgba(0.29, 0.41, 0.50, 0.5),
                    directional_light_color: Color::rgba_u8(255, 238, 227, 127),
                    directional_light_exponent: 30.0,
                    falloff: FogFalloff::Linear {
                        start: 100.0,
                        end: 1000.0,
                    },
                },
            ));

            next_state.set(GameClientState::Playing);
        }
    }
}

pub fn build_client_net_config(
    client_id: u64,
    server_details: &str,
) -> lightyear::prelude::client::NetConfig {
    let server_addr = server_details
        .parse()
        .expect("Unable to parse socket address");
    let auth = lightyear::prelude::client::Authentication::Manual {
        server_addr,
        client_id,
        private_key: [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        protocol_id: 1,
    };
    let client_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0);
    let transport_config = TransportConfig::WebTransportClient {
        client_addr,
        server_addr,
    };

    return client::NetConfig::Netcode {
        auth,
        config: client::NetcodeConfig::default(),
        io: IoConfig::from_transport(transport_config),
    };
}

fn get_local_client_id(net_config: &NetConfig) -> Option<u64> {
    if let NetConfig::Netcode {
        auth,
        config: _,
        io: _,
    } = net_config
    {
        if let Authentication::Manual {
            server_addr: _,
            client_id,
            private_key: _,
            protocol_id: _,
        } = auth
        {
            return Some(*client_id);
        }
    }

    None
}
