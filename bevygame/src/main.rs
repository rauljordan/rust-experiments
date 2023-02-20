use bevy::{prelude::*, render::view::RenderLayers};

fn main() {
    App::new()
        // .add_plugins(DefaultPlugins.set(WindowPlugin {
        //     window: WindowDescriptor {
        //         // fill the entire browser window
        //         fit_canvas_to_parent: true,
        //         ..default()
        //     },
        //     ..default()
        // }))
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.))) // <-- new
        .add_startup_system(setup)
        .add_system(animate_sprite)
        .add_system(move_player)
        .run();
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = (sprite.index + 1) % texture_atlas.textures.len();
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let p = "/home/zodiark/Desktop/code/data/48x48/Roki_idle_48x48.png";
    let texture_handle = asset_server.load(p.to_string());
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(48.0, 96.0), 4, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands.spawn(Camera2dBundle::default());

    let background_image: Handle<Image> =
        asset_server.load("/home/zodiark/Desktop/code/data/Japanese_Home_1_preview_48x48.png");

    commands.spawn((
        RenderLayers::layer(0),
        SpriteBundle {
            texture: background_image,
            transform: Transform::from_scale(Vec3::new(1., 1., 0.0)),
            ..Default::default()
        },
    ));

    commands.spawn((
        Player,
        RenderLayers::layer(0),
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle.clone(),
            transform: Transform::from_scale(Vec3::new(1., 1., 0.0))
                .with_translation(Vec3::new(40., 40., 100.)),
            ..Default::default()
        },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}

#[derive(Component)]
struct Player;

fn move_player(keys: Res<Input<KeyCode>>, mut player_query: Query<&mut Transform, With<Player>>) {
    let mut direction = Vec2::ZERO;
    if keys.any_pressed([KeyCode::Up, KeyCode::W]) {
        direction.y += 3.;
    }
    if keys.any_pressed([KeyCode::Down, KeyCode::S]) {
        direction.y -= 3.;
    }
    if keys.any_pressed([KeyCode::Right, KeyCode::D]) {
        direction.x += 3.;
    }
    if keys.any_pressed([KeyCode::Left, KeyCode::A]) {
        direction.x -= 3.;
    }
    if direction == Vec2::ZERO {
        return;
    }

    let move_speed = 0.8;
    let move_delta = (direction * move_speed).extend(0.);

    for mut transform in player_query.iter_mut() {
        transform.translation += move_delta;
    }
}
