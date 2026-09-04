use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowMode},
};
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
        .add_systems(Update, (keyboard_input, update_player, reset_world).chain())
        .add_observer(toggle_camera)
        .add_observer(toggle_fullscreen)
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
        Collider::cylinder(0.5, 100.0),
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
                    Mesh3d(meshes.add(Cuboid::from_length(1.0))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: colors[k],
                        //alpha_mode: AlphaMode::Add,
                        emissive: LinearRgba::from(colors[k]),
                        emissive_exposure_weight: 0.8,
                        ..default()
                    })),
                    Resetable::from_xyz(i as f32 * 8.0, 1.0 + k as f32 * 1.5, j as f32 * 8.0),
                    RigidBody::Dynamic,
                    Collider::cuboid(0.5, 0.5, 0.5),
                    Restitution::coefficient(1.1),
                    Velocity::zero(),
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
            Velocity::zero(),
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

fn keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut input: ResMut<PlayerInput>,
) {
    let up = keyboard.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) as i8;
    let down = keyboard.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) as i8;
    let left = keyboard.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) as i8;
    let right = keyboard.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) as i8;
    input.throttle = (up - down).into();
    input.steering = (right - left).into();
    if keyboard.just_pressed(KeyCode::Space) {
        input.jump = true;
    }
    if keyboard.just_pressed(KeyCode::KeyR) {
        input.reset = true;
    }
    if keyboard.just_pressed(KeyCode::F5) {
        commands.trigger(ToggleCamera)
    }
    if keyboard.just_pressed(KeyCode::F11) {
        commands.trigger(ToggleFullscreen)
    }
}

fn update_player(
    time: Res<Time>,
    mut input: ResMut<PlayerInput>,
    mut query: Query<(&Player, &Transform, &mut Velocity)>,
) {
    for (_player, transform, mut velocity) in &mut query {
        let dt = time.delta_secs();
        if input.throttle == 0.0 {
            velocity.linear.x *= 0.9;
            velocity.linear.z *= 0.9;
        } else {
            let delta_v = -transform.local_z() * input.throttle * LINEAR_ACCELERATION * dt;
            velocity.linear.x += delta_v.x;
            velocity.linear.z += delta_v.z;
        }

        if input.steering == 0.0 {
            velocity.angular.y *= 0.7;
        } else {
            velocity.angular.y += -input.steering * ANGULAR_ACCELERATION * dt;
            velocity.angular.y = velocity
                .angular
                .y
                .clamp(-MAX_ANGULAR_SPEED, MAX_ANGULAR_SPEED);
        }

        if input.jump {
            input.jump = false; // Clear the jump command
            velocity.linear.y = JUMP_IMPULSE;
        }

        if transform.translation.y < -10.0 {
            println!("fell off world");
            input.reset = true;
        }
    }
}

fn reset_world(
    mut input: ResMut<PlayerInput>,
    mut query: Query<(&Resetable, &mut Transform, &mut Velocity)>,
) {
    if input.reset {
        println!("reset");
        input.reset = false;
        for (resetable, mut transform, mut velocity) in &mut query {
            transform.translation.x = resetable.origin.x;
            transform.translation.y = resetable.origin.y;
            transform.translation.z = resetable.origin.z;
            transform.rotation.x = 0.0;
            transform.rotation.y = 0.0;
            transform.rotation.z = 0.0;
            velocity.linear.x = 0.0;
            velocity.linear.y = 0.0;
            velocity.linear.z = 0.0;
            velocity.angular.x = 0.0;
            velocity.angular.y = 0.0;
            velocity.angular.z = 0.0;
        }
    }
}

fn toggle_camera(
    _on: On<ToggleCamera>,
    mut input: ResMut<PlayerInput>,
    mut query: Query<(&Camera3d, &mut Transform)>,
) {
    input.camera_first_person = !input.camera_first_person;
    for (_camera, mut transform) in &mut query {
        *transform = if input.camera_first_person {
            Transform::from_xyz(0.0, 1.0, 0.0).looking_at(Vec3::new(0.0, 1.0, -1.0), Vec3::Y)
        } else {
            Transform::from_xyz(0.0, 3.0, 13.0).looking_at(Vec3::ZERO, Vec3::Y)
        };
    }
}

fn toggle_fullscreen(
    _on: On<ToggleFullscreen>,
    mut query: Query<&mut Window, With<PrimaryWindow>>,
) {
    println!("fullscreen");
    if let Ok(mut window) = query.single_mut() {
        window.mode = match window.mode {
            WindowMode::Windowed => WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
            _ => WindowMode::Windowed,
        };
    }
}
