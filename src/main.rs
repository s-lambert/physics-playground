use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

const WINDOW_SIZE: f32 = 500.0;

fn setup_scene_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-3.0, 3.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
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
        Collider::cuboid(10.0, 0.1, 10.0),
    ));

    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(1.6, 1.0, 3.0))),
            material: materials.add(Color::AZURE.into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::cuboid(0.8, 0.5, 1.5),
        ExternalImpulse::default(),
    ));
}

fn bump_character(keyboard_input: Res<Input<KeyCode>>, mut query: Query<&mut ExternalImpulse>) {
    let Some(mut impulses) = query.iter_mut().next() else { return };

    if keyboard_input.just_pressed(KeyCode::Up) {
        let impulse = Vec3::new(0.0, 10.0, 0.0);
        let front_left = Vec3::new(0.8, -0.5, 1.5);
        let fl_torque = front_left.cross(impulse);
        let front_right = Vec3::new(-0.8, -0.5, 1.5);
        let fr_torque = front_right.cross(impulse);
        let back_left = Vec3::new(0.8, -0.5, -1.5);
        let bl_torque = back_left.cross(impulse);
        let back_right = Vec3::new(-0.8, -0.5, -1.5);
        let br_torque = back_right.cross(impulse);
        impulses.impulse = impulse;
        impulses.torque_impulse = fl_torque + fr_torque + bl_torque + br_torque;
    }
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
        .add_system(bump_character)
        .run();
}
