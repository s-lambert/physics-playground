use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

const WINDOW_SIZE: f32 = 500.0;

#[derive(Component)]
struct Car;

#[derive(Component)]
struct WheelPoints {
    direction: String,
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

    // Ground
    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(200.0, 0.2, 200.0))),
            material: materials.add(Color::SILVER.into()),
            transform: Transform::from_xyz(0.0, -0.1, 0.0),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(100.0, 0.1, 100.0),
    ));

    // Bump
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

    // Ramp
    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(20.0, 0.2, 20.0))),
            material: materials.add(Color::SILVER.into()),
            transform: Transform {
                translation: Vec3::new(0.0, 2.1, 30.0),
                rotation: Quat::from_axis_angle(Vec3::X, -PI / 8.0),
                ..default()
            },
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(10.0, 0.1, 10.0),
    ));

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 3.0, -10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    commands
        .spawn((
            Car,
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
                linear_damping: 2.0,
                angular_damping: 5.0,
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

fn get_y_rotation(quat: Quat) -> Quat {
    let euler = quat.to_euler(EulerRot::YXZ);
    Quat::from_euler(EulerRot::YXZ, euler.0, 0.0, 0.0)
}

fn move_car(
    time: Res<Time>,
    rapier_context: Res<RapierContext>,
    keyboard_input: Res<Input<KeyCode>>,
    mut car_query: Query<
        (&Transform, &mut Velocity, &mut ExternalForce),
        (With<Car>, Without<WheelPoints>),
    >,
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

    let mut wheel_hits = 0.0;
    let mut wheel_normals = Vec3::default();

    for wheel_point in wheel_points_query.iter() {
        let wheel_translation = wheel_point.translation();
        if let Some((_entity, intersection)) = rapier_context.cast_ray_and_get_normal(
            wheel_translation,
            ray_dir,
            max_toi,
            solid,
            query_filter,
        ) {
            wheel_hits += 1.0;
            wheel_normals += intersection.normal;

            let strength = 1.0 - intersection.toi / max_toi;
            let wheel_force = suspension_force * strength;
            let wheel_torque = (wheel_translation - car_translation).cross(wheel_force) * strength;
            total_torque += wheel_torque;
            total_force += wheel_force;
        }
    }

    let driving_plane = if wheel_hits > 0.0 {
        Quat::from_rotation_arc(Vec3::Y, wheel_normals / wheel_hits)
    } else {
        Quat::from_rotation_arc(Vec3::Y, Vec3::Y)
    };
    let driving_force =
        driving_plane * get_y_rotation(car_transform.rotation) * Vec3::new(0.0, 0.0, 160.0);

    if keyboard_input.pressed(KeyCode::Up) {
        let acceleration_point = car_transform.rotation * Vec3::new(0.0, -0.1, 0.1);
        let acceleration_torque =
            (acceleration_point).cross(car_transform.rotation * Vec3::new(0.0, 0.0, 160.0));
        total_force += driving_force;
        total_torque += acceleration_torque;
    } else if keyboard_input.pressed(KeyCode::Down) {
        let braking_point = car_transform.rotation * Vec3::new(0.0, -0.1, -0.1);
        let braking_torque =
            (braking_point).cross(-(car_transform.rotation * Vec3::new(0.0, 0.0, 160.0)));
        total_force += -driving_force;
        total_torque += braking_torque;
    }

    if keyboard_input.pressed(KeyCode::Left) {
        velocity.angvel.y += 20.0 * time.delta_seconds();
    } else if keyboard_input.pressed(KeyCode::Right) {
        velocity.angvel.y -= 20.0 * time.delta_seconds();
    }
    velocity.angvel.y = velocity.angvel.y.clamp(-20.0, 20.0);

    forces.force = total_force;
    forces.torque = total_torque;
}

fn camera_follow(
    mut camera_transform_query: Query<&mut Transform, With<Camera>>,
    car_transform_query: Query<&Transform, (With<Car>, Without<Camera>)>,
) {
    let Some(mut camera_transform) = camera_transform_query.iter_mut().next() else { return };
    let Some(car_transform) = car_transform_query.iter().next() else { return };

    camera_transform.translation = car_transform.translation
        + get_y_rotation(car_transform.rotation) * Vec3::new(0.0, 3.0, -10.0);
    camera_transform.rotation =
        Quat::from_axis_angle(Vec3::Y, PI) * get_y_rotation(car_transform.rotation);
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        title: "Physics Playground!".to_string(),
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
        .add_startup_system(setup_scene)
        .add_system(move_car)
        .add_system(camera_follow)
        .run();
}
