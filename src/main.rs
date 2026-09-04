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

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-3.0, 3.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Collider::cuboid(100.0, 0.1, 100.0),
        Transform::from_xyz(0.0, -2.0, 0.0),
    ));

    commands.spawn((
        RigidBody::Dynamic,
        Collider::ball(0.5),
        Restitution::coefficient(1.7),
        Transform::from_xyz(0.0, 4.0, 0.0),
    ));
}

fn print_ball_altitude(query: Query<&Transform, With<RigidBody>>) {
    for transform in &query {
        println!("Ball altitude: {}", transform.translation.y);
    }
}
