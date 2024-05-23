use std::time::Duration;
use std::vec;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy::window::{PrimaryWindow, WindowPlugin};
use rand::prelude::random;

const SNAKE_HEAD_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const SNAKE_SEGMENT_COLOR: Color = Color::rgb(0.3, 0.3, 0.3);
const ARENA_WIDTH: u32 = 10;
const ARENA_HEIGHT: u32 = 10;
const FOOD_COLOR: Color = Color::rgb(1.0, 0.0, 1.0);

#[derive(Component)]
struct SnakeHead {
    direction: Direction,
}

#[derive(Component)]
struct SnakeSegment;

#[derive(Default, Resource)]
struct SnakeSegments(Vec<Entity>);

#[derive(PartialEq, Clone, Copy)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Resource, Default)]
struct LastTailPosition(Option<Position>);

#[derive(Event)]
struct GrowthEvent;

#[derive(Event)]
struct GameOverEvent;

#[derive(Component)]
struct Food;

#[derive(Component)]
struct Size {
    width: f32,
    height: f32,
}

impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "game muito pika mane".to_string(),
                resolution: (500.0, 500.0).into(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .insert_resource(SnakeSegments::default())
        .insert_resource(LastTailPosition::default())
        .add_event::<GrowthEvent>()
        .add_event::<GameOverEvent>()
        .add_systems(Startup, (spawn_snake, setup_camera))
        .add_systems(
            Update,
            (
                snake_growth.after(snake_eating),
                snake_eating.after(snake_movement),
                game_over.after(snake_movement),
                snake_input_moviment.before(snake_movement),
                snake_movement.run_if(on_timer(Duration::from_secs_f32(0.150))),
                food_spawner.run_if(on_timer(Duration::from_secs(1))),
            ),
        )
        .add_systems(PostUpdate, (position_translation, size_scaling))
        .run();
}

fn setup_camera(mut command: Commands) {
    command.spawn(Camera2dBundle::default());
}

fn spawn_snake(mut commands: Commands, mut segments: ResMut<SnakeSegments>) {
    let head = commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: SNAKE_HEAD_COLOR,
                    ..Default::default()
                },
                ..Default::default()
            },
            SnakeHead {
                direction: Direction::Up,
            },
            SnakeSegment,
            Position { x: 3, y: 3 },
            Size::square(0.8),
        ))
        .id();

    let segment = spawn_segment(commands, Position { x: 3, y: 2 });
    *segments = SnakeSegments(vec![head, segment]);
}

fn spawn_segment(mut command: Commands, position: Position) -> Entity {
    command
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: SNAKE_SEGMENT_COLOR,
                    ..Default::default()
                },
                ..Default::default()
            },
            SnakeSegment,
            position,
            Size::square(0.65),
        ))
        .id()
}

fn snake_eating(
    mut command: Commands,
    mut growth_writter: EventWriter<GrowthEvent>,
    food_position: Query<(Entity, &Position), With<Food>>,
    head_position: Query<&Position, With<SnakeHead>>,
) {
    for head_pos in head_position.iter() {
        for (ent, food_pos) in food_position.iter() {
            if food_pos == head_pos {
                command.entity(ent).despawn();
                growth_writter.send(GrowthEvent);
            }
        }
    }
}

fn snake_movement(
    segments: ResMut<SnakeSegments>,
    mut heads: Query<(Entity, &SnakeHead)>,
    mut last_tail_position: ResMut<LastTailPosition>,
    mut game_over_writer: EventWriter<GameOverEvent>,
    mut positions: Query<&mut Position>,
) {
    if let Some((head_entity, head)) = heads.iter_mut().next() {
        let segment_positions: Vec<Position> = segments
            .0
            .iter()
            .map(|&e| *positions.get_mut(e).unwrap())
            .collect();

        let mut head_pos = positions.get_mut(head_entity).unwrap();
        match &head.direction {
            Direction::Left => head_pos.x -= 1,
            Direction::Right => head_pos.x += 1,
            Direction::Up => head_pos.y += 1,
            Direction::Down => head_pos.y -= 1,
        }

        if head_pos.x < 0
            || head_pos.y < 0
            || head_pos.x as u32 >= ARENA_WIDTH
            || head_pos.y as u32 >= ARENA_HEIGHT
        {
            game_over_writer.send(GameOverEvent);
        }

        if segment_positions.contains(&head_pos) {
            game_over_writer.send(GameOverEvent);
        }

        // Update the positions of the rest of the segments
        for (pos, segment) in segment_positions.iter().zip(segments.0.iter().skip(1)) {
            let mut segment_pos = positions.get_mut(*segment).unwrap();
            *segment_pos = *pos;
        }

        *last_tail_position = LastTailPosition(Some(*segment_positions.last().unwrap()));
    }
}

fn snake_growth(
    command: Commands,
    last_tail_position: ResMut<LastTailPosition>,
    mut segments: ResMut<SnakeSegments>,
    mut growth_reader: EventReader<GrowthEvent>,
) {
    if growth_reader.read().into_iter().next().is_some() {
        segments
            .0
            .push(spawn_segment(command, last_tail_position.0.unwrap()));
    }
}

fn snake_input_moviment(input: Res<ButtonInput<KeyCode>>, mut heads: Query<&mut SnakeHead>) {
    if let Some(mut head) = heads.iter_mut().next() {
        let dir: Direction = if input.pressed(KeyCode::ArrowLeft) {
            Direction::Left
        } else if input.pressed(KeyCode::ArrowRight) {
            Direction::Right
        } else if input.pressed(KeyCode::ArrowDown) {
            Direction::Down
        } else if input.pressed(KeyCode::ArrowUp) {
            Direction::Up
        } else {
            head.direction
        };

        if dir != head.direction.opposite() {
            head.direction = dir;
        }
    }
}

fn game_over(
    mut command: Commands,
    mut reader: EventReader<GameOverEvent>,
    segments: ResMut<SnakeSegments>,
    food: Query<Entity, With<Food>>,
    segment: Query<Entity, With<SnakeSegment>>,
) {
    if reader.read().into_iter().next().is_some() {
        for ent in food.iter().chain(segment.iter()) {
            command.entity(ent).despawn();
        }

        spawn_snake(command, segments);
    }
}

fn size_scaling(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut q: Query<(&Size, &mut Transform)>,
) {
    if let Ok(window) = windows.get_single_mut() {
        for (sprite_size, mut transform) in q.iter_mut() {
            transform.scale = Vec3::new(
                sprite_size.width / ARENA_WIDTH as f32 * window.width(),
                sprite_size.height / ARENA_HEIGHT as f32 * window.height(),
                1.0,
            );
        }
    }
}

fn food_spawner(mut command: Commands) {
    command.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: FOOD_COLOR,
                ..Default::default()
            },
            ..Default::default()
        },
        Food,
        Position {
            x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
            y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
        },
        Size::square(0.8),
    ));
}

fn position_translation(
    mut windows: Query<&Window, With<PrimaryWindow>>,
    mut q: Query<(&Position, &mut Transform)>,
) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }
    if let Ok(window) = windows.get_single_mut() {
        for (pos, mut transform) in q.iter_mut() {
            transform.translation = Vec3::new(
                convert(pos.x as f32, window.width(), ARENA_WIDTH as f32),
                convert(pos.y as f32, window.height(), ARENA_HEIGHT as f32),
                0.0,
            );
        }
    }
}
