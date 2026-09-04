use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

const LINEAR_ACCELERATION: f32 = 30.0; // m/s^2
const MAX_LINEAR_SPEED: f32 = 30.0; // m/s
const ANGULAR_ACCELERATION: f32 = 6.5; // radians/s^2
const MAX_ANGULAR_SPEED: f32 = 2.5; // radians/s
const JUMP_IMPULSE: f32 = 7.0; // m/s
const USER: f32 = 1.3; // m [Size of the player block]

#[derive(Component, Deref)]
struct Resetable {
    origin: Vec3,
}

impl Resetable {
    fn from_xyz(x: f32, y: f32, z: f32) -> (Self, Transform) {
        (
            Resetable {
                origin: Vec3::new(x, y, z),
            },
            Transform::from_xyz(x, y, z),
        )
    }
}

#[derive(Component)]
struct Player;

#[derive(Resource, Deref, DerefMut, Debug)]
struct PlayerInput {
    #[deref]
    throttle: f32,
    steering: f32,
    jump: bool,
    reset: bool,
    camera_first_person: bool,
}

#[derive(Event)]
struct ToggleCamera;

#[derive(Event)]
struct ToggleFullscreen;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
        ))
        .insert_resource(PlayerInput {
            throttle: 0.0,
            steering: 0.0,
            jump: false,
            reset: false,
            camera_first_person: false,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, print_ball_altitude)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // The floor
    commands.spawn((
        RigidBody::Fixed,
        Collider::cylinder(1.0, 100.0),
        Mesh3d(meshes.add(Cylinder::new(100.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.7, 0.9),
            ..default()
        })),
        Transform::from_xyz(0.0, -1.0, 0.0),
    ));

    // That bouncey ball
    commands.spawn((
        RigidBody::Dynamic,
        Collider::ball(0.5),
        Restitution::coefficient(1.7),
        Transform::from_xyz(4.0, 4.0, 0.0),
    ));

    // Some lights!
    for i in -1..2 {
        for j in -1..2 {
            commands.spawn((
                PointLight {
                    shadow_maps_enabled: true,
                    contact_shadows_enabled: true,
                    intensity: 7_000_000.0,
                    ..default()
                },
                Transform::from_xyz(i as f32 * 35.0, 11.0, j as f32 * 35.0),
            ));
        }
    }

    // Those stacks of colorfull cubes
    let colors = [
        Color::srgb_u8(124, 144, 255),
        Color::srgb_u8(240, 255, 124),
        Color::srgb_u8(124, 255, 144),
        Color::srgb_u8(255, 144, 124),
        Color::srgb_u8(124, 144, 255),
        Color::srgb_u8(240, 255, 124),
        Color::srgb_u8(124, 255, 144),
        Color::srgb_u8(255, 144, 124),
    ];
    for i in -2..3_i32 {
        for j in -2..3_i32 {
            for k in 0..8 {
                commands.spawn((
                    RigidBody::Dynamic,
                    Collider::cuboid(0.5, 0.5, 0.5),
                    Restitution::coefficient(1.1),
                    Mesh3d(meshes.add(Cuboid::from_length(1.0))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: colors[k],
                        //alpha_mode: AlphaMode::Add,
                        emissive: LinearRgba::from(colors[k]),
                        emissive_exposure_weight: 0.8,
                        ..default()
                    })),
                    Resetable::from_xyz(i as f32 * 8.0, 1.0 + k as f32 * 1.5, j as f32 * 8.0),
                ));
            }
        }
    }

    // The "player" (a brown cube)
    commands
        .spawn((
            Player,
            Mesh3d(meshes.add(Cuboid::from_length(USER))),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
            Resetable::from_xyz(0.0, 15.0, 0.0),
            RigidBody::Dynamic,
            Collider::cuboid(USER / 2.0, USER / 2.0, USER / 2.0),
            Restitution::coefficient(1.1),
            LockedAxes::ROTATION_LOCKED,
        ))
        .with_children(|parent| {
            parent.spawn((
                Camera3d::default(),
                Transform::from_xyz(0.0, 3.0, 13.0).looking_at(Vec3::ZERO, Vec3::Y),
            ));
        });
}

fn print_ball_altitude(query: Query<&Transform, With<RigidBody>>) {
    for transform in &query {
        println!("Ball altitude: {}", transform.translation.y);
    }
}
