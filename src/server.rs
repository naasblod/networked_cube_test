use bevy::app::ScheduleRunnerPlugin;
use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;

use std::net::{Ipv4Addr, SocketAddr};

use bevy::render::settings::WgpuSettings;
use bevy::render::RenderPlugin;
use bevy::winit::WinitPlugin;
use bevy_tnua::control_helpers::TnuaCrouchEnforcerPlugin;
use bevy_tnua::controller::{TnuaController, TnuaControllerBundle, TnuaControllerPlugin};
use bevy_tnua::TnuaUserControlsSystemSet;
use bevy_tnua_xpbd3d::{TnuaXpbd3dPlugin, TnuaXpbd3dSensorShape};
use bevy_xpbd_3d::components::{LinearVelocity, LockedAxes, Position, RigidBody};
use bevy_xpbd_3d::plugins::collision::Collider;
use bevy_xpbd_3d::plugins::setup::Physics;
use bevy_xpbd_3d::plugins::PhysicsPlugins;
use leafwing_input_manager::action_state::ActionState;
use lightyear::server::input_leafwing::LeafwingInputPlugin;
use lightyear::shared::config::Mode;
use lightyear::{prelude::*, server::events::MessageEvent};

use async_compat::Compat;
use bevy::tasks::IoTaskPool;
use bevy::tasks::TaskPool;
use lightyear::prelude::server::Certificate;

use std::collections::HashMap;
use std::time::Duration;

use crate::movement::shared_movement_behaviour;
use crate::protocol::{
    protocol, ClientAssetLoadingComplete, MyProtocol, PlayerActions, PlayerBundle, Replicate,
};
use crate::shared::{shared_config, SharedPlugin, FIXED_TIMESTEP_HZ};

#[derive(Resource)]
pub(crate) struct ServerGlobal {
    pub client_id_to_entity_id: HashMap<ClientId, Entity>,
}

pub fn server_app(net_config: server::NetConfig) -> App {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(LogPlugin {
                update_subscriber: None,
                level: Level::INFO,
                filter: "wgpu=error,symphonia_core=error,symphonia_format_ogg=error".to_string(),
            })
            .set(RenderPlugin {
                render_creation: WgpuSettings {
                    backends: None,
                    ..default()
                }
                .into(),
                ..default()
            })
            .disable::<WinitPlugin>()
            .disable::<GilrsPlugin>(),
    );

    app.add_plugins(ScheduleRunnerPlugin::default());

    let server_config = server::ServerConfig {
        shared: shared_config(Mode::Separate),
        net: vec![net_config],
        // replication: server::ReplicationConfig {
        //     // enable send because we pre-spawn entities on the client
        //     enable_send: true,
        //     enable_receive: true,
        // },
        ..default()
    };

    let plugin_config = server::PluginConfig::new(server_config, protocol());

    app.add_plugins(lightyear::prelude::server::ServerPlugin::new(plugin_config));

    app.insert_resource(ServerGlobal {
        client_id_to_entity_id: Default::default(),
    });
    app.add_systems(Startup, init);
    app.add_systems(
        Update,
        (handle_connections, on_client_asset_loading_complete),
    )
    .add_plugins(LeafwingInputPlugin::<MyProtocol, PlayerActions>::default());

    app.add_plugins(SharedPlugin);

    app
}

pub fn build_server_net_config() -> lightyear::prelude::server::NetConfig {
    let certificate = IoTaskPool::get_or_init(|| TaskPool::new())
        .scope(|s| {
            s.spawn(Compat::new(async {
                Certificate::load("certificates/cert.pem", "certificates/key.pem")
                    .await
                    .unwrap()
            }));
        })
        .pop()
        .unwrap();
    let digest = &certificate.hashes()[0].to_string().replace(":", "");
    println!("Certificate with digest: {}", digest);

    let netcode_config = server::NetcodeConfig::default()
        .with_protocol_id(1)
        .with_key([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);

    let transport_config = TransportConfig::WebTransportServer {
        server_addr: SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 5000),
        certificate,
    };

    server::NetConfig::Netcode {
        config: netcode_config,
        io: IoConfig::from_transport(transport_config).with_conditioner(LinkConditionerConfig {
            incoming_latency: Duration::from_millis(100),
            incoming_jitter: Default::default(),
            incoming_loss: 0.0,
        }),
    }
}

fn movement(
    tick_manager: Res<TickManager>,
    mut action_query: Query<(
        Entity,
        &Position,
        &mut TnuaController,
        &ActionState<PlayerActions>,
    )>,
) {
    for (entity, position, mut controller, action_state) in action_query.iter_mut() {
        // NOTE: be careful to directly pass Mut<PlayerPosition>
        // getting a mutable reference triggers change detection, unless you use `as_deref_mut()`
        // shared_movement_behaviour(velocity, action);
        info!(?entity, tick = ?tick_manager.tick(), ?position, actions = ?action_state.get_pressed(), "applying movement to player");

        shared_movement_behaviour(&mut controller, action_state);
    }
}

fn init(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut connections: ResMut<lightyear::prelude::server::ServerConnections>,
) {
    connections.start().expect("Failed to start server");

    // Spawn the ground.
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(128.0, 128.0)),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        },
        RigidBody::Static,
        Collider::halfspace(Vec3::Y),
    ));
}

fn handle_connections(
    // mut connections: EventReader<lightyear::prelude::server::ConnectEvent>,
    mut disconnections: EventReader<lightyear::prelude::server::DisconnectEvent>,
    mut global: ResMut<ServerGlobal>,
    mut commands: Commands,
) {
    for disconnection in disconnections.read() {
        let client_id = disconnection.context();
        // TODO: handle this automatically in lightyear
        //  - provide a Owned component in lightyear that can specify that an entity is owned by a specific player?
        //  - maybe have the client-id to entity-mapping in the global metadata?
        //  - despawn automatically those entities when the client disconnects
        if let Some(entity) = global.client_id_to_entity_id.remove(client_id) {
            if let Some(mut entity) = commands.get_entity(entity) {
                entity.despawn();
            }
        }
    }
}

fn on_client_asset_loading_complete(
    mut reader: EventReader<MessageEvent<ClientAssetLoadingComplete>>,
    mut global: ResMut<ServerGlobal>,
    mut commands: Commands,
) {
    for event in reader.read() {
        let client_id = *event.context();

        info!(
            "Received ClientLoadingCompleteRequest message: {:?}, {:?}",
            event.message(),
            client_id
        );

        let mut replicate = Replicate {
            prediction_target: NetworkTarget::Single(client_id),
            replicate_hierarchy: false,
            interpolation_target: NetworkTarget::AllExceptSingle(client_id),
            ..default()
        };

        replicate
            .add_target::<ActionState<PlayerActions>>(NetworkTarget::AllExceptSingle(client_id));

        replicate.add_target::<LinearVelocity>(NetworkTarget::Single(client_id));

        let entity = commands.spawn((
            PlayerBundle::new(client_id, Vec3::new(0.0, 10.0, 0.0)),
            replicate,
            TnuaControllerBundle::default(),
            // LockedAxes::new().lock_rotation_x().lock_rotation_z(),
            LockedAxes::ROTATION_LOCKED,
            TnuaXpbd3dSensorShape(Collider::cuboid(0.98, 0.98, 0.98)),
            SpatialBundle {
                transform: Transform::from_xyz(0.0, 10.0, 0.0),
                ..default()
            },
        ));

        // Add a mapping from client id to entity id
        global.client_id_to_entity_id.insert(client_id, entity.id());
        info!("Create entity {:?} for client {:?}", entity.id(), client_id);
    }
}
