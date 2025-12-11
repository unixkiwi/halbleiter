use bevy::audio::Volume;
use bevy::math::ops::round;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::window::WindowResolution;
use rand::prelude::SliceRandom;
use rand::rng;
use std::cmp::min;
use std::collections::HashSet;

const WIN_TITLE: &str = "Halbleiter";

const WIN_WIDTH: u32 = 1500;
const WIN_HEIGHT: u32 = 720;

const BACKGROUND_COLOR: Color = Color::srgb(0.1, 0.1, 0.1);

fn main() {
    let window = WindowPlugin {
        primary_window: Some(Window {
            title: WIN_TITLE.to_string(),
            resolution: WindowResolution::new(WIN_WIDTH, WIN_HEIGHT),
            resizable: false,
            ..default()
        }),
        ..default()
    };

    // let grid = grid::grid![
    //     [Some(Tile::Cable {entry: Side::Right, exit: Side::Bottom}), Some(Tile::Battery {plus_side: Side::Left, minus_side: Side::Right}), Some(Tile::N)]
    //     [Some(Tile::Cable {entry: Side::Top, exit: Side::Right}), Some(Tile::Lamp {entry: Side::Left, exit: Side::Bottom}), Some(Tile::P)]
    //     [None, Some(Tile::Cable {entry: Side::Top, exit: Side::Right}), Some(Tile::Cable {entry: Side::Left, exit: Side::Top})]
    // ];

    /*App::new()
    .add_plugins(DefaultPlugins
        .set(window)
        .set(ImagePlugin::default_nearest())
    )
    .insert_resource(ClearColor(BACKGROUND_COLOR))
    .insert_resource(Grid(grid![]))
    .add_systems(Startup, setup)
    .add_systems(PostStartup, |mut commands: Commands| commands.trigger(MakeNewPuzzleRequest))
    .add_systems(Update, (tile_drag_system, restart_listener))
    .add_observer(new_puzzle)
    .run();*/


    App::new()
        .add_plugins(
            DefaultPlugins
                .set(window)
                .set(ImagePlugin::default_nearest()),
        )
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .init_state::<AppState>()
        .add_systems(Startup, setup_camera)
        // Menu Systems
        .add_systems(OnEnter(AppState::Menu), spawn_menu)
        .add_systems(OnExit(AppState::Menu), cleanup_menu)
        // Game Systems
        .add_systems(OnEnter(AppState::Game), (setup).chain())
        .add_systems(
            Update,
            (tile_drag_system, restart_listener).run_if(in_state(AppState::Game)),
        )
        .add_systems(PostStartup, |mut commands: Commands| {
            commands.trigger(MakeNewPuzzleRequest)
        })
        .add_observer(new_puzzle)
        .run();
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum AppState {
    #[default]
    Menu,
    Game,
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// -------------------------------------------------------------------------------------------------
// MENU
// -------------------------------------------------------------------------------------------------

#[derive(Component)]
#[require(Node, BackgroundColor)]
struct MenuRoot;

fn spawn_menu(mut commands: Commands) {
    let root = commands
        .spawn((
            MenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
        ))
        .id();

    // Title Text
    let title = commands
        .spawn((
            Text::new("Main Menu"),
            TextFont {
                font_size: 60.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ))
        .id();

    // Play Button
    let play_button = commands
        .spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(65.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            BorderColor::all(Color::BLACK),
            BorderRadius::all(Val::Px(10.0)),
        ))
        .observe(
            |_trigger: On<Pointer<Click>>, mut next_state: ResMut<NextState<AppState>>| {
                info!("Play button clicked!");
                next_state.set(AppState::Game);
            },
        )
        .with_children(|parent| {
            parent.spawn((
                Text::new("Play Game"),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        })
        .id();

    // Quit Button
    let quit_button = commands
        .spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(65.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.5, 0.1, 0.1)),
            BorderRadius::all(Val::Px(10.0)),
        ))
        .observe(|_: On<Pointer<Click>>, mut exit: MessageWriter<AppExit>| {
            exit.write(AppExit::Success);
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new("Quit"),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        })
        .id();

    // Build screen hierarchy
    commands
        .entity(root)
        .add_children(&[title, play_button, quit_button]);
}

// 4. Cleanup System
fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MenuRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

// -------------------------------------------------------------------------------------------------
// GAME
// -------------------------------------------------------------------------------------------------
#[derive(Resource)]
struct Sounds {
    drop: Handle<AudioSource>,
    start_drag: Handle<AudioSource>,
    lamp_turns_on: Handle<AudioSource>,
    misdrop: Handle<AudioSource>,
}

#[derive(Component)]
#[require(Sprite, Transform)]
struct TileComponent {
    x: usize,
    y: usize,
}

#[derive(Component)]
struct GridLine;

#[derive(Copy, Clone, Debug)]
enum Side {
    Left,
    Right,
    Bottom,
    Top,
}
impl Side {
    fn x_offset(&self) -> i32 {
        match self {
            Side::Left => -1,
            Side::Right => 1,
            Side::Bottom => 0,
            Side::Top => 0,
        }
    }

    fn y_offset(&self) -> i32 {
        match self {
            Side::Left => 0,
            Side::Right => 0,
            Side::Bottom => 1,
            Side::Top => -1,
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum Tile {
    Lamp { entry: Side, exit: Side },
    Battery { plus_side: Side, minus_side: Side },
    Cable { entry: Side, exit: Side },
    P,
    N,
}

#[derive(Resource, Clone, Debug)]
struct Grid(grid::Grid<Option<Tile>>);
impl Grid {
    fn width(&self) -> usize {
        self.0.cols()
    }

    fn height(&self) -> usize {
        self.0.rows()
    }

    fn get(&self, x: usize, y: usize) -> Option<&Option<Tile>> {
        self.0.get(y, x)
    }

    fn swap(&mut self, (x1, y1): (usize, usize), (x2, y2): (usize, usize)) {
        self.0.swap((y1, x1), (y2, x2));
    }

    fn tile_size(&self) -> f32 {
        min(
            WIN_WIDTH / self.width() as u32,
            WIN_HEIGHT / self.height() as u32,
        ) as f32
    }

    fn start_x(&self) -> f32 {
        -(self.width() as f32) / 2.0 * self.tile_size()
    }

    fn start_y(&self) -> f32 {
        (self.height() as f32) / 2.0 * self.tile_size()
    }

    fn world_pos_from_xy(&self, x: usize, y: usize) -> Option<Vec2> {
        if x >= self.width() || y >= self.height() {
            return None;
        }

        Some(Vec2::new(
            self.start_x() + x as f32 * self.tile_size(),
            self.start_y() - y as f32 * self.tile_size(),
        ))
    }

    fn xy_from_world_pos(&self, pos: Vec2) -> (usize, usize) {
        let mut normalized = (pos - Vec2::new(self.start_x(), self.start_y())) / self.tile_size();

        normalized.x = round(normalized.x);
        normalized.y = round(normalized.y);

        if normalized.x < 0.0 {
            normalized.x = 0.0;
        }
        if normalized.y > 0.0 {
            normalized.y = 0.0;
        }
        if normalized.x >= self.width() as f32 {
            normalized.x = self.width() as f32 - 1.0;
        }
        if normalized.y <= -(self.height() as f32) {
            normalized.y = self.height() as f32 + 1.0;
        }

        (normalized.x as usize, -normalized.y as usize)
    }

    fn snap_to_grid(&self, pos: Vec2) -> Vec2 {
        let start = Vec2::new(self.start_x(), self.start_y());

        let mut normalized = (pos - start) / self.tile_size();
        normalized.x = round(normalized.x);
        normalized.y = round(normalized.y);

        if normalized.x < 0.0 {
            normalized.x = 0.0;
        }
        if normalized.y > 0.0 {
            normalized.y = 0.0;
        }
        if normalized.x >= self.width() as f32 {
            normalized.x = self.width() as f32 - 1.0;
        }
        if normalized.y <= -(self.height() as f32) {
            normalized.y = self.height() as f32 + 1.0;
        }

        normalized * self.tile_size() + start
    }

    fn has_unobstructed_path(&self, (x1, y1): (usize, usize), (x2, y2): (usize, usize)) -> bool {
        if y1 == y2 && (x1 + 1 == x2 || x2 + 1 == x1) {
            return true;
        }

        if x1 == x2 && (y1 + 1 == y2 || y2 + 1 == y1) {
            return true;
        }

        if let Some(x) = self.get(x1 + 1, y1)
            && x.is_none()
        {
            return self.has_unobstructed_path((x1 + 1, y1), (x2, y2));
        }

        if let Some(x) = self.get(x1, y1 + 1)
            && x.is_none()
        {
            return self.has_unobstructed_path((x1, y1 + 1), (x2, y2));
        }

        if let Some(x) = self.get(x2 + 1, y2)
            && x.is_none()
        {
            return self.has_unobstructed_path((x1, y1), (x2 + 1, y2));
        }

        if let Some(x) = self.get(x2, y2 + 1)
            && x.is_none()
        {
            return self.has_unobstructed_path((x1, y1), (x2, y2 + 1));
        }

        false
    }

    fn is_solved_helper(
        &self,
        start_x: i32,
        start_y: i32,
        prev_x: i32,
        prev_y: i32,
        prev_tile: Tile,
        found_lamp_init: bool,
    ) -> bool {
        #[derive(Hash, Eq, PartialEq, Copy, Clone)]
        struct StateKey {
            x: i32,
            y: i32,
            prev_x: i32,
            prev_y: i32,
            prev_kind: u8, // tile type compressed
            found_lamp: bool,
        }

        fn tile_kind(t: Tile) -> u8 {
            match t {
                Tile::Battery { .. } => 0,
                Tile::Lamp { .. } => 1,
                Tile::Cable { .. } => 2,
                Tile::P => 3,
                Tile::N => 4,
            }
        }

        let mut visited: HashSet<StateKey> = HashSet::new();

        let mut stack = vec![(start_x, start_y, prev_x, prev_y, prev_tile, found_lamp_init)];

        while let Some((x, y, prev_x, prev_y, prev_tile, found_lamp)) = stack.pop() {
            // State key
            let key = StateKey {
                x,
                y,
                prev_x,
                prev_y,
                prev_kind: tile_kind(prev_tile),
                found_lamp,
            };

            // Skip if visited
            if !visited.insert(key) {
                continue;
            }

            // Bounds
            if x < 0 || x >= self.width() as i32 || y < 0 || y >= self.height() as i32 {
                continue;
            }

            let tile = match self.get(x as usize, y as usize).unwrap() {
                Some(t) => t,
                None => continue,
            };

            match tile {
                Tile::Lamp { entry, exit } => {
                    if (x + entry.x_offset(), y + entry.y_offset()) != (prev_x, prev_y) {
                        continue;
                    }
                    stack.push((x + exit.x_offset(), y + exit.y_offset(), x, y, *tile, true));
                }

                Tile::Cable { entry, exit } => {
                    if (x + entry.x_offset(), y + entry.y_offset()) != (prev_x, prev_y) {
                        continue;
                    }
                    stack.push((
                        x + exit.x_offset(),
                        y + exit.y_offset(),
                        x,
                        y,
                        *tile,
                        found_lamp,
                    ));
                }

                Tile::Battery {
                    plus_side: _,
                    minus_side,
                } => {
                    if (x + minus_side.x_offset(), y + minus_side.y_offset()) != (prev_x, prev_y) {
                        continue;
                    }
                    if found_lamp {
                        return true;
                    }
                }

                Tile::P => {
                    for side in [Side::Left, Side::Right, Side::Top, Side::Bottom] {
                        let nx = x + side.x_offset();
                        let ny = y + side.y_offset();

                        if nx < 0 || ny < 0 {
                            continue;
                        }

                        if let Some(Some(Tile::N)) = self.get(nx as usize, ny as usize) {
                            stack.push((nx, ny, x, y, *tile, found_lamp));
                        }
                    }
                }

                Tile::N => {
                    if !matches!(prev_tile, Tile::P) {
                        continue;
                    }
                    for side in [Side::Left, Side::Right, Side::Top, Side::Bottom] {
                        let nx = x + side.x_offset();
                        let ny = y + side.y_offset();
                        stack.push((nx, ny, x, y, *tile, found_lamp));
                    }
                }
            }
        }

        false
    }

    fn is_solved(&self) -> bool {
        let mut found_battery = false;
        let (mut battery_x, mut battery_y, mut plus_side, mut minus_side) =
            (0, 0, Side::Left, Side::Right);
        'outer: for x in 0..self.width() {
            for y in 0..self.height() {
                if let Some(Tile::Battery {
                    plus_side: plus,
                    minus_side: minus,
                }) = self.get(x, y).unwrap()
                {
                    (battery_x, battery_y, plus_side, minus_side) = (x, y, *plus, *minus);
                    found_battery = true;
                    break 'outer;
                }
            }
        }

        if !found_battery {
            return false;
        }

        self.is_solved_helper(
            battery_x as i32 + plus_side.x_offset(),
            battery_y as i32 + plus_side.y_offset(),
            battery_x as i32,
            battery_y as i32,
            Tile::Battery {
                plus_side,
                minus_side,
            },
            false,
        )
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load Sounds
    commands.insert_resource(Sounds {
        drop: asset_server.load("audio/drop.wav"),
        start_drag: asset_server.load("audio/start_drag.wav"),
        lamp_turns_on: asset_server.load("audio/lamp_on2.wav"),
        misdrop: asset_server.load("audio/misdrop.wav"),
    });

    // Restart Text
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: px(5),
            left: px(10),
            ..default()
        },
        Text::new("Druecke 'R' um ein neues Puzzle\nzu generieren"),
        TextFont {
            font_size: 20.0,
            ..Default::default()
        },
    ));
}

fn get_path_to_start_sprite_for_tile(tile: &Tile) -> &'static str {
    match tile {
        Tile::P => "sprites/p.png",
        Tile::N => "sprites/n.png",

        Tile::Lamp { entry, exit } => match (entry, exit) {
            (Side::Right, Side::Bottom) => "sprites/lamp_off_right_to_bottom.png",
            (Side::Right, Side::Left) => "sprites/lamp_off_right_to_left.png",
            (Side::Right, Side::Top) => "sprites/lamp_off_right_to_top.png",

            (Side::Bottom, Side::Right) => "sprites/lamp_off_bottom_to_right.png",
            (Side::Bottom, Side::Left) => "sprites/lamp_off_bottom_to_left.png",
            (Side::Bottom, Side::Top) => "sprites/lamp_off_bottom_to_top.png",

            (Side::Left, Side::Right) => "sprites/lamp_off_left_to_right.png",
            (Side::Left, Side::Bottom) => "sprites/lamp_off_left_to_bottom.png",
            (Side::Left, Side::Top) => "sprites/lamp_off_left_to_top.png",

            (Side::Top, Side::Right) => "sprites/lamp_off_top_to_right.png",
            (Side::Top, Side::Bottom) => "sprites/lamp_off_top_to_bottom.png",
            (Side::Top, Side::Left) => "sprites/lamp_off_top_to_left.png",

            _ => panic!(),
        },

        Tile::Battery {
            plus_side,
            minus_side,
        } => match (plus_side, minus_side) {
            (Side::Right, Side::Bottom) => "sprites/battery_plus_right_minus_bottom.png",
            (Side::Right, Side::Left) => "sprites/battery_plus_right_minus_left.png",
            (Side::Right, Side::Top) => "sprites/battery_plus_right_minus_top.png",

            (Side::Bottom, Side::Right) => "sprites/battery_plus_bottom_minus_right.png",
            (Side::Bottom, Side::Left) => "sprites/battery_plus_bottom_minus_left.png",
            (Side::Bottom, Side::Top) => "sprites/battery_plus_bottom_minus_top.png",

            (Side::Left, Side::Right) => "sprites/battery_plus_left_minus_right.png",
            (Side::Left, Side::Bottom) => "sprites/battery_plus_left_minus_bottom.png",
            (Side::Left, Side::Top) => "sprites/battery_plus_left_minus_top.png",

            (Side::Top, Side::Right) => "sprites/battery_plus_top_minus_right.png",
            (Side::Top, Side::Bottom) => "sprites/battery_plus_top_minus_bottom.png",
            (Side::Top, Side::Left) => "sprites/battery_plus_top_minus_left.png",

            _ => panic!(),
        },

        Tile::Cable { entry, exit } => match (entry, exit) {
            (Side::Right, Side::Bottom) => "sprites/cable_right_to_bottom.png",
            (Side::Right, Side::Left) => "sprites/cable_right_to_left.png",
            (Side::Right, Side::Top) => "sprites/cable_right_to_top.png",

            (Side::Bottom, Side::Right) => "sprites/cable_bottom_to_right.png",
            (Side::Bottom, Side::Left) => "sprites/cable_bottom_to_left.png",
            (Side::Bottom, Side::Top) => "sprites/cable_bottom_to_top.png",

            (Side::Left, Side::Right) => "sprites/cable_left_to_right.png",
            (Side::Left, Side::Bottom) => "sprites/cable_left_to_bottom.png",
            (Side::Left, Side::Top) => "sprites/cable_left_to_top.png",

            (Side::Top, Side::Right) => "sprites/cable_top_to_right.png",
            (Side::Top, Side::Bottom) => "sprites/cable_top_to_bottom.png",
            (Side::Top, Side::Left) => "sprites/cable_top_to_left.png",

            _ => panic!(),
        },
    }
}

fn get_path_to_lamp_on_sprite_for_tile(tile: &Tile) -> &'static str {
    let Tile::Lamp { entry, exit } = tile else {
        unreachable!()
    };
    match (entry, exit) {
        (Side::Right, Side::Bottom) => "sprites/lamp_on_right_to_bottom.png",
        (Side::Right, Side::Left) => "sprites/lamp_on_right_to_left.png",
        (Side::Right, Side::Top) => "sprites/lamp_on_right_to_top.png",

        (Side::Bottom, Side::Right) => "sprites/lamp_on_bottom_to_right.png",
        (Side::Bottom, Side::Left) => "sprites/lamp_on_bottom_to_left.png",
        (Side::Bottom, Side::Top) => "sprites/lamp_on_bottom_to_top.png",

        (Side::Left, Side::Right) => "sprites/lamp_on_left_to_right.png",
        (Side::Left, Side::Bottom) => "sprites/lamp_on_left_to_bottom.png",
        (Side::Left, Side::Top) => "sprites/lamp_on_left_to_top.png",

        (Side::Top, Side::Right) => "sprites/lamp_on_top_to_right.png",
        (Side::Top, Side::Bottom) => "sprites/lamp_on_top_to_bottom.png",
        (Side::Top, Side::Left) => "sprites/lamp_on_top_to_left.png",

        _ => panic!(),
    }
}

#[derive(Event)]
struct MakeNewPuzzleRequest;

fn cleanup_game(
    mut commands: Commands,
    tiles: Query<Entity, With<TileComponent>>,
    grid_lines: Query<Entity, With<GridLine>>,
) {
    // Delete previous tiles
    for entity in tiles.iter() {
        commands.entity(entity).despawn();
    }

    // Delete previous grid lines
    for entity in grid_lines.iter() {
        commands.entity(entity).despawn();
    }
}

fn new_puzzle(
    _event: On<MakeNewPuzzleRequest>,
    mut commands: Commands,
    tiles: Query<(Entity, &TileComponent)>,
    grid_lines: Query<Entity, With<GridLine>>,
    asset_server: Res<AssetServer>,
) {
    // Delete Previous tiles
    for (entity, _) in tiles.iter() {
        commands.entity(entity).despawn();
    }

    // Delete Previous grid lines
    for entity in grid_lines.iter() {
        commands.entity(entity).despawn();
    }

    // Create New
    let grid = generate_puzzle();
    commands.insert_resource(grid.clone());

    // UI
    let tile_size = Vec2::new(grid.tile_size(), grid.tile_size());

    // Lines
    let thickness = 1.0;
    let grid_pixel_w = grid.width() as f32 * tile_size.x;
    let grid_pixel_h = grid.height() as f32 * tile_size.y;
    let start = Vec2::new(grid.start_x(), grid.start_y());

    // Vertical lines
    for i in 0..=grid.width() {
        let x = start.x + i as f32 * tile_size.x;
        commands.spawn((
            GridLine,
            Anchor::TOP_LEFT,
            Sprite {
                color: Color::linear_rgb(0.75, 0.75, 0.75),
                custom_size: Some(Vec2::new(thickness, grid_pixel_h)),
                ..default()
            },
            Transform::from_translation(Vec3::new(x - thickness / 2.0, start.y, -1.0)),
        ));
    }

    // Horizontal lines
    for j in 0..=grid.height() {
        let y = start.y - j as f32 * tile_size.y;
        commands.spawn((
            GridLine,
            Anchor::TOP_LEFT,
            Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(grid_pixel_w, thickness)),
                ..default()
            },
            Transform::from_translation(Vec3::new(start.x, y - thickness / 2.0, -1.0)),
        ));
    }

    // Tiles
    for x in 0..grid.width() {
        for y in 0..grid.height() {
            let tile = grid.get(x, y).unwrap();
            let Some(tile) = tile else {
                continue;
            };

            let sprite_path = get_path_to_start_sprite_for_tile(tile);
            let sprite = asset_server.load(sprite_path);

            let pos = grid.world_pos_from_xy(x, y).unwrap();

            commands.spawn((
                TileComponent { x, y },
                Anchor::TOP_LEFT,
                Sprite {
                    image: sprite,
                    custom_size: Some(tile_size),
                    ..default()
                },
                Transform::from_translation(Vec3::new(pos.x, pos.y, 0.0)),
            ));
        }
    }
}

fn generate_puzzle() -> Grid {
    let possible_tiles = [
        vec![
            Some(Tile::Cable {
                entry: Side::Right,
                exit: Side::Bottom,
            }),
            Some(Tile::Battery {
                plus_side: Side::Left,
                minus_side: Side::Right,
            }),
            Some(Tile::N),
            Some(Tile::Cable {
                entry: Side::Top,
                exit: Side::Right,
            }),
            Some(Tile::Lamp {
                entry: Side::Left,
                exit: Side::Bottom,
            }),
            Some(Tile::P),
            None,
            Some(Tile::Cable {
                entry: Side::Top,
                exit: Side::Right,
            }),
            Some(Tile::Cable {
                entry: Side::Left,
                exit: Side::Top,
            }),
        ],
        vec![
            Some(Tile::N),
            Some(Tile::P),
            None,
            Some(Tile::Cable {
                entry: Side::Top,
                exit: Side::Bottom,
            }),
            Some(Tile::Battery {
                plus_side: Side::Top,
                minus_side: Side::Right,
            }),
            Some(Tile::Lamp {
                entry: Side::Bottom,
                exit: Side::Left,
            }),
            Some(Tile::Cable {
                entry: Side::Top,
                exit: Side::Right,
            }),
            Some(Tile::Cable {
                entry: Side::Left,
                exit: Side::Right,
            }),
            Some(Tile::Cable {
                entry: Side::Left,
                exit: Side::Top,
            }),
        ],
        vec![
            Some(Tile::N),
            Some(Tile::P),
            None,
            Some(Tile::Cable {
                entry: Side::Top,
                exit: Side::Bottom,
            }),
            Some(Tile::Cable {
                entry: Side::Bottom,
                exit: Side::Right,
            }),
            Some(Tile::Cable {
                entry: Side::Left,
                exit: Side::Top,
            }),
            Some(Tile::Cable {
                entry: Side::Bottom,
                exit: Side::Left,
            }),
            Some(Tile::Lamp {
                entry: Side::Right,
                exit: Side::Left,
            }),
            Some(Tile::Battery {
                plus_side: Side::Bottom,
                minus_side: Side::Right,
            }),
        ],
        vec![
            None,
            Some(Tile::N),
            Some(Tile::P),
            Some(Tile::Battery {
                plus_side: Side::Left,
                minus_side: Side::Top,
            }),
            Some(Tile::Cable {
                entry: Side::Right,
                exit: Side::Left,
            }),
            Some(Tile::Lamp {
                entry: Side::Right,
                exit: Side::Top,
            }),
            Some(Tile::Cable {
                entry: Side::Bottom,
                exit: Side::Right,
            }),
            Some(Tile::Cable {
                entry: Side::Left,
                exit: Side::Top,
            }),
            Some(Tile::Cable {
                entry: Side::Bottom,
                exit: Side::Right,
            }),
        ],
    ];

    let mut tiles = possible_tiles[rand::random_range(..possible_tiles.len())].clone();
    tiles.shuffle(&mut rng());
    let grid = grid::Grid::from_vec(tiles, 3);

    Grid(grid)
}

fn restart_listener(input: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if input.just_pressed(KeyCode::KeyR) {
        commands.trigger(MakeNewPuzzleRequest);
    }
}

struct TileDragSystemCurrent {
    entity: Entity,
    offset_from_cursor: Vec2,
    start_pos: Vec3,
}
#[derive(Default, Component)]
struct TileDragSystemState {
    cursor_world_pos: Vec2,
    current: Option<TileDragSystemCurrent>,
}

#[allow(clippy::too_many_arguments)]
fn tile_drag_system(
    mut state: Local<TileDragSystemState>,
    mut cursor_moved_event_reader: MessageReader<CursorMoved>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut tiles: Query<(Entity, &mut TileComponent, &mut Sprite)>,
    mut transforms: Query<&mut Transform>,
    mut grid: ResMut<Grid>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    sounds: Res<Sounds>,
) {
    // Update cursor position
    let half_window = Vec2::new(WIN_WIDTH as f32 / 2.0, WIN_HEIGHT as f32 / 2.0);
    if let Some(cursor_event) = cursor_moved_event_reader.read().last() {
        state.cursor_world_pos = cursor_event.position - half_window;
        state.cursor_world_pos.y *= -1.0;
    };

    // Drop
    if mouse_button_input.just_released(MouseButton::Left)
        && let Some(current) = &state.current
    {
        let mut sprite_pos = transforms.get_mut(current.entity).unwrap();
        let (start_x, start_y) = grid.xy_from_world_pos(current.start_pos.xy());

        // Checks
        let (new_x, new_y) = grid.xy_from_world_pos(sprite_pos.translation.xy());
        let is_new_cell_empty = grid.get(new_x, new_y).unwrap().is_none();
        let has_unobstructed_path = grid.has_unobstructed_path((new_x, new_y), (start_x, start_y));

        if is_new_cell_empty && has_unobstructed_path {
            // Snap
            let snapped = grid.snap_to_grid(sprite_pos.translation.xy());
            sprite_pos.translation = Vec3::new(snapped.x, snapped.y, 0.0);

            // Update Grid
            grid.swap((new_x, new_y), (start_x, start_y));

            let mut tile = tiles.get_mut(current.entity).unwrap().1;
            tile.x = new_x;
            tile.y = new_y;

            // Solved?
            let is_solved = grid.is_solved();

            for (_, tile, mut sprite) in tiles.iter_mut() {
                if let Some(lamp @ Tile::Lamp { .. }) = grid.get(tile.x, tile.y).unwrap() {
                    sprite.image = asset_server.load(match is_solved {
                        true => get_path_to_lamp_on_sprite_for_tile(lamp),
                        false => get_path_to_start_sprite_for_tile(lamp),
                    });
                }
            }

            if is_solved {
                // Audio
                commands.spawn((
                    AudioPlayer::new(sounds.lamp_turns_on.clone()),
                    PlaybackSettings::DESPAWN,
                ));
            }

            // Audio
            commands.spawn((AudioPlayer::new(sounds.drop.clone()), {
                let mut settings = PlaybackSettings::DESPAWN;
                settings.volume = Volume::Linear(0.25);
                settings
            }));
        } else {
            sprite_pos.translation = current.start_pos.truncate().extend(0.0);

            // Audio
            commands.spawn((AudioPlayer::new(sounds.misdrop.clone()), {
                let mut settings = PlaybackSettings::DESPAWN;
                settings.volume = Volume::Linear(0.2);
                settings
            }));
        }

        state.current = None;
        return;
    }

    // Drag
    if mouse_button_input.pressed(MouseButton::Left)
        && let Some(current) = &state.current
    {
        let mut sprite_pos = transforms.get_mut(current.entity).unwrap();

        sprite_pos.translation.x = state.cursor_world_pos.x + current.offset_from_cursor.x;
        sprite_pos.translation.y = state.cursor_world_pos.y + current.offset_from_cursor.y;
        sprite_pos.translation.z = 10.0;
    }

    // Start drag
    if mouse_button_input.just_pressed(MouseButton::Left) {
        for (entity, _, sprite) in tiles.iter_mut() {
            let sprite_pos = transforms
                .get_mut(entity)
                .unwrap()
                .translation
                .truncate()
                .extend(10.0);
            let cursor_pos = state.cursor_world_pos;

            let sprite_size = sprite.custom_size.unwrap();

            if cursor_pos.x >= sprite_pos.x
                && cursor_pos.x <= sprite_pos.x + sprite_size.x
                && cursor_pos.y <= sprite_pos.y
                && cursor_pos.y >= sprite_pos.y - sprite_size.y
            {
                state.current = Some(TileDragSystemCurrent {
                    entity,
                    offset_from_cursor: Vec2::new(
                        sprite_pos.x - cursor_pos.x,
                        sprite_pos.y - cursor_pos.y,
                    ),
                    start_pos: sprite_pos,
                });

                // Audio
                commands.spawn((AudioPlayer::new(sounds.start_drag.clone()), {
                    let mut settings = PlaybackSettings::DESPAWN;
                    settings.volume = Volume::Linear(0.25);
                    settings
                }));
            }
        }
    }
}
