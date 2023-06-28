use bevy::prelude::*;
use bevy::sprite::collide_aabb::{collide, Collision};
use bevy::sprite::MaterialMesh2dBundle;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use leafwing_input_manager::prelude::*;

pub const HEIGHT: f32 = 720.0;
pub const WIDTH: f32 = 1200.0;

pub const FIELD_WIDTH: f32 = WIDTH;
pub const FIELD_HEIGHT: f32 = HEIGHT - 120.0;

pub const MIN_PADDLE_Y: f32 = -1.0 * ((HEIGHT / 2.0) - 5.0 - 75.0);
pub const MAX_PADDLE_Y: f32 = HEIGHT / 2.0 - 120.0 - 5.0 - 75.0;

pub const MAX_PADDLE_SPEED: f32 = 300.0;
pub const MIN_PADDLE_SPEED: f32 = -1.0 * MAX_PADDLE_SPEED;

pub const PADDLE_SIZE: Vec3 = Vec3::new(20.0, 150.0, 1.0);
pub const INITIAL_BALL_VELOCITY: Vec2 = Vec2::new(200.0, 200.0);
pub const INITIAL_BALL_POSITION: Vec3 = Vec3::new(0.0, -60.0, 0.0);

pub const SCOREBOARD_FONT_SIZE: f32 = 48.0;
pub const SCORE_COLOR: Color = Color::CYAN;
pub const SCOREBOARD_LEFT: Val = Val::Px(10.0);
pub const SCOREBOARD_MIDDLE: Val = Val::Px(600.0);
pub const SCOREBOARD_RIGHT: Val = Val::Px(1200.0);
pub const SCOREBOARD_TOP: Val = Val::Px(10.0);
pub const SCOREBOARD_BOTTOM: Val = Val::Px(110.0);

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Opponent;

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub struct Goal;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Paddle;

#[derive(Component)]
pub struct Ball;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Velocity {
    linvel: Vec2,
}

#[derive(Component)]
pub struct Collider;

#[derive(Resource)]
pub struct Score {
    player: i32,
    cpu: i32,
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum Action {
    Up,
    Down,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .add_startup_system(spawn_basic_scene)
        .add_startup_system(spawn_camera)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (WIDTH, HEIGHT).into(),
                title: "Pong".to_string(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugin(InputManagerPlugin::<Action>::default())
        // .add_plugin(WorldInspectorPlugin::new())
        .register_type::<Paddle>()
        .add_system(player_input)
        .add_system(opponent_input)
        .add_systems((
            check_for_collisions,
            score_goal.after(check_for_collisions),
            apply_velocity.before(check_for_collisions),
            move_paddles
                .before(check_for_collisions)
                .after(apply_velocity),
        ))
        .add_system(update_score)
        .insert_resource(Score { player: 0, cpu: 0 })
        .run();
}

fn spawn_basic_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.25, 0.25, 0.25),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(-500.0, 0.0, 0.0),
                scale: PADDLE_SIZE,
                ..default()
            },
            ..default()
        })
        .insert(Paddle)
        .insert(InputManagerBundle::<Action> {
            action_state: ActionState::default(),
            input_map: InputMap::new([(KeyCode::Up, Action::Up), (KeyCode::Down, Action::Down)]),
        })
        .insert(Velocity {
            linvel: Vec2::new(0.0, 0.0),
        })
        .insert(Collider)
        .insert(Player);

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.25, 0.25, 0.25),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(500.0, 0.0, 0.0),
                scale: PADDLE_SIZE,
                ..default()
            },
            ..default()
        })
        .insert(Paddle)
        .insert(Velocity {
            linvel: Vec2::new(0.0, 0.0),
        })
        .insert(Collider)
        .insert(Opponent);

    commands
        .spawn(MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::default().into()).into(),
            material: materials.add(ColorMaterial::from(Color::PURPLE)),
            transform: Transform::from_translation(INITIAL_BALL_POSITION)
                .with_scale(Vec3::new(20.0, 20.0, 0.0)),
            ..default()
        })
        .insert(Ball)
        .insert(Velocity {
            linvel: INITIAL_BALL_VELOCITY,
        })
        .insert(Name::new("Ball"));

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.75, 0.75, 0.75),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(WIDTH / 2.0, -60.0, 0.0),
                scale: Vec3::new(10.0, FIELD_HEIGHT, 1.0),
                ..default()
            },
            ..default()
        })
        .insert(Goal);

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.75, 0.75, 0.75),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(-1.0 * WIDTH / 2.0, -60.0, 0.0),
                scale: Vec3::new(10.0, FIELD_HEIGHT, 1.0),
                ..default()
            },
            ..default()
        })
        .insert(Goal);

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.75, 0.75, 0.75),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 240.0, 0.0),
                scale: Vec3::new(FIELD_WIDTH, 10.0, 1.0),
                ..default()
            },
            ..default()
        })
        .insert(Collider)
        .insert(Wall);

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.75, 0.75, 0.75),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, -360.0, 0.0),
                scale: Vec3::new(FIELD_WIDTH, 10.0, 1.0),
                ..default()
            },
            ..default()
        })
        .insert(Collider)
        .insert(Wall);

    commands
        .spawn(
            TextBundle::from_sections([
                TextSection {
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: SCOREBOARD_FONT_SIZE,
                        color: SCORE_COLOR,
                        ..default()
                    },
                    value: "you   ".to_string(),
                },
                TextSection {
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: SCOREBOARD_FONT_SIZE,
                        color: SCORE_COLOR,
                        ..default()
                    },
                    value: "0".to_string(),
                },
            ])
            .with_style(Style {
                position_type: PositionType::Absolute,
                position: UiRect::new(
                    SCOREBOARD_LEFT,
                    SCOREBOARD_MIDDLE,
                    SCOREBOARD_TOP,
                    SCOREBOARD_BOTTOM,
                ),
                ..default()
            }),
        )
        .insert(Player);
    commands
        .spawn(
            TextBundle::from_sections([
                TextSection {
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: SCOREBOARD_FONT_SIZE,
                        color: SCORE_COLOR,
                        ..default()
                    },
                    value: "0".to_string(),
                },
                TextSection {
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: SCOREBOARD_FONT_SIZE,
                        color: SCORE_COLOR,
                        ..default()
                    },

                    value: "   cpu".to_string(),
                },
            ])
            .with_style(Style {
                position_type: PositionType::Absolute,
                position: UiRect::new(
                    Val::Px(1000.0),
                    SCOREBOARD_RIGHT,
                    SCOREBOARD_TOP,
                    SCOREBOARD_BOTTOM,
                ),
                ..default()
            }),
        )
        .insert(Opponent);
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn opponent_input(
    mut opponent: Query<(&mut Velocity, &Transform), With<Opponent>>,
    ball: Query<&Transform, (With<Ball>, Without<Opponent>)>,
) {
    let (mut paddle_velocity, opponent_transform) = opponent.single_mut();
    let ball_transform = ball.single();
    let speed = 100.0;
    if ball_transform.translation.y < opponent_transform.translation.y {
        paddle_velocity.linvel.y = -1.0 * speed;
    } else if ball_transform.translation.y > opponent_transform.translation.y {
        paddle_velocity.linvel.y = speed;
    } else {
        paddle_velocity.linvel.y = 0.0;
    }
}

fn player_input(mut query: Query<(&ActionState<Action>, &mut Velocity), With<Player>>) {
    let (action_state, mut paddle_velocity) = query.single_mut();

    if action_state.pressed(Action::Up) {
        paddle_velocity.linvel.y += 100.0;
    } else if action_state.pressed(Action::Down) {
        paddle_velocity.linvel.y -= 100.0;
    }
    paddle_velocity.linvel.y = paddle_velocity
        .linvel
        .y
        .clamp(MIN_PADDLE_SPEED, MAX_PADDLE_SPEED);
}

fn apply_velocity(mut ball: Query<(&mut Transform, &Velocity), With<Ball>>, time: Res<Time>) {
    let (mut ball_pos, ball_vel) = ball.single_mut();
    ball_pos.translation.x += ball_vel.linvel.x * time.delta_seconds();
    ball_pos.translation.y += ball_vel.linvel.y * time.delta_seconds();
}

fn move_paddles(mut paddles: Query<(&mut Transform, &Velocity), With<Paddle>>, time: Res<Time>) {
    for (mut paddle_pos, paddle_vel) in paddles.iter_mut() {
        paddle_pos.translation.y += paddle_vel.linvel.y * time.delta_seconds();
        paddle_pos.translation.y = paddle_pos.translation.y.clamp(MIN_PADDLE_Y, MAX_PADDLE_Y);
    }
}

fn check_for_collisions(
    mut ball: Query<(&mut Velocity, &Transform), With<Ball>>,
    colliders: Query<&Transform, With<Collider>>,
) {
    let (mut ball_vel, ball_pos) = ball.single_mut();
    let ball_size = ball_pos.scale.truncate();

    for collider_pos in colliders.iter() {
        let collision = collide(
            ball_pos.translation,
            ball_size,
            collider_pos.translation,
            collider_pos.scale.truncate(),
        );
        if let Some(collision) = collision {
            let mut reflect_x = false;
            let mut reflect_y = false;
            match collision {
                Collision::Left => reflect_x = ball_vel.linvel.x > 0.0,
                Collision::Right => reflect_x = ball_vel.linvel.x < 0.0,
                Collision::Top => reflect_y = ball_vel.linvel.y < 0.0,
                Collision::Bottom => reflect_y = ball_vel.linvel.y > 0.0,
                Collision::Inside => {}
            }
            if reflect_x {
                ball_vel.linvel.x *= -1.0;
            }
            if reflect_y {
                ball_vel.linvel.y *= -1.0;
            }
        }
    }
}

fn score_goal(
    mut ball: Query<&mut Transform, With<Ball>>,
    goals: Query<&Transform, (Without<Ball>, With<Goal>)>,
    mut score: ResMut<Score>,
) {
    let mut ball_pos = ball.single_mut();
    let ball_size = ball_pos.scale.truncate();

    for collider_pos in goals.iter() {
        let collision = collide(
            ball_pos.translation,
            ball_size,
            collider_pos.translation,
            collider_pos.scale.truncate(),
        );

        if let Some(collision) = collision {
            ball_pos.translation = INITIAL_BALL_POSITION;
            if collision == Collision::Left {
                score.player += 1;
            }
            if collision == Collision::Right {
                score.cpu += 1;
            }
        }
    }
}

fn update_score(
    mut player_score: Query<&mut Text, With<Player>>,
    mut cpu_score: Query<&mut Text, (With<Opponent>, Without<Player>)>,
    score: Res<Score>,
) {
    let mut player_text = player_score.single_mut();
    let mut cpu_text = cpu_score.single_mut();

    player_text.sections[1].value = score.player.to_string();
    cpu_text.sections[0].value = score.cpu.to_string();
}
