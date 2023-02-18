use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

const WINDOW_SIZE: f32 = 500.0;

#[derive(Component)]
struct WheelPoints {
    direction: String,
}

fn setup_scene_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-3.0, 3.0, -10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });

    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(20.0, 0.2, 20.0))),
            material: materials.add(Color::SILVER.into()),
            transform: Transform::from_xyz(0.0, -0.1, 0.0),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(10.0, 0.1, 10.0),
    ));

    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(20.0, 0.2, 20.0))),
            material: materials.add(Color::SILVER.into()),
            transform: Transform::from_xyz(0.0, 0.1, 20.0),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(10.0, 0.1, 10.0),
    ));

    commands
        .spawn((
            MaterialMeshBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(1.6, 1.0, 3.0))),
                material: materials.add(Color::AZURE.into()),
                transform: Transform::from_xyz(0.0, 2.0, 0.0),
                ..default()
            },
            RigidBody::Dynamic,
            Velocity::default(),
            Collider::cuboid(0.8, 0.5, 1.5),
            ExternalForce::default(),
            // This damping is to stabilise the wheel forces, might need to be implemented when calculating the force
            Damping {
                linear_damping: 5.0,
                angular_damping: 1.0,
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                WheelPoints {
                    direction: "Front Left".to_string(),
                },
                TransformBundle::from_transform(Transform::from_translation(Vec3::new(
                    0.8, -0.5, 1.5,
                ))),
            ));
            parent.spawn((
                WheelPoints {
                    direction: "Front Right".to_string(),
                },
                TransformBundle::from_transform(Transform::from_translation(Vec3::new(
                    -0.8, -0.5, 1.5,
                ))),
            ));
            parent.spawn((
                WheelPoints {
                    direction: "Back Left".to_string(),
                },
                TransformBundle::from_transform(Transform::from_translation(Vec3::new(
                    0.8, -0.5, -1.5,
                ))),
            ));
            parent.spawn((
                WheelPoints {
                    direction: "Back Right".to_string(),
                },
                TransformBundle::from_transform(Transform::from_translation(Vec3::new(
                    -0.8, -0.5, -1.5,
                ))),
            ));
        });
}

fn move_car(
    rapier_context: Res<RapierContext>,
    keyboard_input: Res<Input<KeyCode>>,
    mut car_query: Query<(&Transform, &mut Velocity, &mut ExternalForce), Without<WheelPoints>>,
    wheel_points_query: Query<&GlobalTransform, With<WheelPoints>>,
) {
    let Some((car_transform, mut velocity, mut forces)) = car_query.iter_mut().next() else { return };
    let car_translation = car_transform.translation;

    let suspension_force = Vec3::new(0.0, 18.0, 0.0);

    let mut total_force = Vec3::default();
    let mut total_torque: Vec3 = Vec3::default();

    let ray_dir = Vec3::NEG_Y;
    let max_toi = 1.0;
    let solid = true;
    let query_filter = QueryFilter::only_fixed();

    for wheel_point in wheel_points_query.iter() {
        let wheel_translation = wheel_point.translation();
        if let Some((_entity, toi)) =
            rapier_context.cast_ray(wheel_translation, ray_dir, max_toi, solid, query_filter)
        {
            let strength = 1.0 - toi / max_toi;
            let wheel_force = suspension_force * strength;
            let wheel_torque = (wheel_translation - car_translation).cross(wheel_force) * strength;
            total_torque += wheel_torque;
            total_force += wheel_force;
        }
    }

    let driving_force = Vec3::new(0.0, 0.0, 20.0);

    if keyboard_input.pressed(KeyCode::Up) {
        total_force += driving_force;
    } else if keyboard_input.pressed(KeyCode::Down) {
        total_force -= driving_force;
    }

    forces.force = total_force;
    forces.torque = total_torque;
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        title: "Monster Survivors!".to_string(),
                        width: WINDOW_SIZE,
                        height: WINDOW_SIZE,
                        ..default()
                    },
                    ..default()
                }),
        )
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_system(bevy::window::close_on_esc)
        .add_startup_system(setup_scene_camera)
        .add_startup_system(setup_scene)
        .add_system(move_car)
        .run();
}
