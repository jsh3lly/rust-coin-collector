use bevy::math::vec2;
use bevy::prelude::*;
// use bevy::window::WindowMode;
// use std::default::Default;
use std::fmt;
use std::fmt::Formatter;
use std::ops::Deref;
use bevy::ecs::bundle::BundleId;
use bevy::input::keyboard::KeyboardInput;
use rand::Rng;
// use heron::prelude::*;
// use heron::rapier_plugin::rapier2d::math::Real;
// use heron::rapier_plugin::rapier2d::prelude::{RigidBodyCcd, RigidBodyType};
use bevy_rapier2d::prelude::*;
use rand::prelude::SliceRandom;
// use crate::KeyCode::Right;
// use crate::CursorIcon::Default;


// Some important initial config
static WINDOW_WIDTH: f32 = 1000 as f32;
static WINDOW_HEIGHT: f32 = 600 as f32;
static SPRITE_SIZE: f32 = 50.0;  // SIZE MUST ALWAYS BE A FACTOR OF WINDOW_WIDTH AND WINDOW_HEIGHT!!
static PLAYSPACE_WIDTH: isize = WINDOW_WIDTH as isize / SPRITE_SIZE as isize;
static PLAYSPACE_HEIGHT: isize = WINDOW_HEIGHT as isize / SPRITE_SIZE as isize;
static MOVESPEED: f32 = 500.;

fn main() {
    App::new()
        // Try to insert the WindowDescriptor at the start itself, otherwise it messes up the scaling etc
        .insert_resource(WindowDescriptor {
            title: "Coin Collector".to_string(),
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            resizable: true,
            mode: bevy::window::WindowMode::Windowed,
            scale_factor_override: Some(1.0),
            ..default()})
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        // .add_plugin(RapierDebugRenderPlugin::default())
           // Adding heron plugin
        // .add_startup_system()
        .init_resource::<GameState>()
        .add_startup_system(spawn_camera)
        // .add_startup_system(spawn_player)
        .add_startup_system(spawn_playspace_entities)
        .add_system(move_player)
        .run();
}

// Resources
#[derive(Clone)]
struct GameState {
    player_spawn_pos: (usize, usize),
    playspace_matrix: PlayspaceMatrix,
}

#[derive(Clone)]
struct PlayspaceMatrix(Vec<Vec<GameObject<'static>>>); //TODO Instead of GameObject, do &GameObject???

impl PlayspaceMatrix {
    fn instantiate_gameobject_at_coord (&mut self, coord : (usize, usize), gameobj_type: GameObjType) {
        match gameobj_type {
            GameObjType::Wall => {self.0[coord.0][coord.1] = GameObject {gameobj_type: GameObjType::Wall, playspace_position: coord, symbol: "W"}}
            GameObjType::Player => {self.0[coord.0][coord.1] = GameObject {gameobj_type: GameObjType::Player, playspace_position: coord, symbol: "P"}}
            Empty => {self.0[coord.0][coord.1] = GameObject {gameobj_type: Empty, playspace_position: coord, symbol: "0"}}
        };
    }

    fn get_gameobj_at_coord(&self, coord : (isize, isize)) ->  Option<&GameObject> {
        let row = self.0.get(coord.0 as usize);
        match row {
            None => return None,
            Some(row) => row.get(coord.1 as usize),
        }
    }

    fn get_neighbours_at_coord(& self, (a,b) :(isize, isize)) {
        let mut possible_neighbour_coords= vec![(a+1,b), (a,b+1), (a+1,b+1)];
        if a-1 >= 0 {
            possible_neighbour_coords.push((a-1,b));
            possible_neighbour_coords.push((a-1,b+1));
        }

        if b-1 >= 0 {
            possible_neighbour_coords.push((a,b-1));
            possible_neighbour_coords.push((a+1,b-1));

        }

        if a-1 >= 0 && b-1 >= 0 {
            possible_neighbour_coords.push((a-1,b-1));
        }

        let mut possible_neighbour_gameobjects : Vec<& GameObject> = vec![];
        for possible_neighbour_coord in possible_neighbour_coords.iter() {
            let possibly_gameobj = self.get_gameobj_at_coord(*possible_neighbour_coord);
            if possibly_gameobj.is_some() { possible_neighbour_gameobjects.push(possibly_gameobj.unwrap()); }
        }

        for gameobj in possible_neighbour_gameobjects.iter() {
            println!("{:?}", gameobj.playspace_position);
        }
    }
}

impl Default for PlayspaceMatrix {
    fn default() -> Self {
        let mut playspacematrix: Self = PlayspaceMatrix(vec![vec![GameObject::default(); PLAYSPACE_WIDTH as usize]; PLAYSPACE_HEIGHT as usize]);
            for i in 0..PLAYSPACE_HEIGHT as usize {
                for j in 0..PLAYSPACE_WIDTH as usize{
                    playspacematrix.instantiate_gameobject_at_coord((i, j), GameObjType::Empty)
                }
            }
            playspacematrix
    }
}

// This is where we 'set' the initial game data!! (we impl Default for all resources)
impl FromWorld for GameState {
    fn from_world(world: &mut World) -> Self {
        let mut game_state = Self {
            player_spawn_pos: (0, 0),
            playspace_matrix: PlayspaceMatrix::default()
        };

        // ==== Walls on the border ====
        // top and bottom
        let playspace_width_usize = PLAYSPACE_WIDTH as usize;
        let playspace_height_usize = PLAYSPACE_HEIGHT as usize;
        let mut coords_border_wall : Vec<(usize, usize)> = Vec::new();
        for i in 0..playspace_width_usize {
            coords_border_wall.push((0, i));
            coords_border_wall.push((playspace_height_usize-1, i));
        }
        // left and right
        for i in 1..playspace_height_usize {
            coords_border_wall.push((i, 0));
            coords_border_wall.push((i, playspace_width_usize-1));
        }

        // actually instantiating those walls
        for (a,b) in coords_border_wall.iter() {
            game_state.playspace_matrix.instantiate_gameobject_at_coord((*a, *b), GameObjType::Wall);
        }
        // ==== Player inside ====
        let mut rng = rand::thread_rng();
        game_state.player_spawn_pos = (rng.gen_range(1..playspace_height_usize-1), rng.gen_range(1..playspace_width_usize-1));
        game_state.playspace_matrix.instantiate_gameobject_at_coord(game_state.player_spawn_pos,
                                                                    GameObjType::Player);


        // Procedurally randomly generated inner walls
        let root_coords_for_generation = coords_border_wall.clone();
        let random_border_wall_coord = root_coords_for_generation.choose(&mut rand::thread_rng()).unwrap(); // These coords would be of a wall
        // game_state.playspace_matrix.0[random_border_wall_coord.0][random_border_wall_coord.1] = "0".to_string();
        game_state.playspace_matrix.instantiate_gameobject_at_coord(*random_border_wall_coord, GameObjType::Empty);

        // game_state.playspace_matrix.get_gameobj_at_coord(*random_border_wall_coord);
        game_state.playspace_matrix.get_neighbours_at_coord((game_state.player_spawn_pos.0 as isize, game_state.player_spawn_pos.0 as isize));
        game_state
    }
}

#[derive(Component)]
struct Player;

#[derive(Clone)]
struct GameObject<'a> {
    gameobj_type: GameObjType,
    playspace_position: (usize, usize),
    symbol: &'a str,
}
//
// impl GameObject {
//     fn generate_neighbour(&self) {
//         let mut possible
//     }
// }

impl Default for GameObject<'_> {
    fn default() -> Self {
        Self {gameobj_type: GameObjType::Empty, playspace_position: (0,0), symbol: "X"}
    }
}

#[derive(Clone, PartialEq)]
enum GameObjType {
    Wall,
    Player,
    Empty,
}


impl GameState {
    fn print_playspace(&self) {
        let cloned_matrix = self.playspace_matrix.clone();
        for row in cloned_matrix.0 {
            print!("[ ");
            for element in row {
                print!("{} ", element.symbol)
            }
            println!("]");

        }
        println!();
    }
}

// Components


// // Custom Bundles
// #[derive(Bundle)]
// struct WallBundle {
//     #[bundle]
//     rigidbody: RigidBody,
//     collider: Collider,
// }
//
// impl Default for WallBundle {
//     fn default() -> Self {
//         Self {
//             rigidbody: RigidBody::Fixed,
//             collider: Collider::cuboid(SPRITE_SIZE / 2.0, SPRITE_SIZE / 2.0),
//         }
//     }
// }



fn spawn_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn move_player(keyboard_input: Res<Input<KeyCode>>, mut player_q: Query<(&mut Velocity, &Player)>) {
    let (mut velocity , player) = player_q.single_mut();
    let mut velocity_vec = Vec2::new(0., 0.);
    let (mut direction_x, mut direction_y) = (0., 0.);
    if keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up){
        direction_y = 1.
    }
    if keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left){
        direction_x = -1.
    }
    if keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down){
        direction_y = -1.
    }
    if keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right){
        direction_x = 1.
    }
    velocity_vec.x = MOVESPEED * direction_x;
    velocity_vec.y = MOVESPEED * direction_y;
    velocity.linvel = velocity_vec;
}

fn spawn_playspace_entities(mut commands: Commands, asset_server: Res<AssetServer>, game_data: Res<GameState>) {
    game_data.print_playspace();
    let playspace_matrix = game_data.playspace_matrix.0.clone();
    for i in 0..playspace_matrix.len() {
        for j in 0..playspace_matrix[0].len() {
            let coords = playspace_coords_to_world_coords((j as isize, i as isize));
            if playspace_matrix[i][j].gameobj_type == GameObjType::Wall {
                commands.spawn().insert_bundle(SpriteBundle {
                sprite: Sprite {custom_size: Option::Some(vec2(SPRITE_SIZE as f32, SPRITE_SIZE as f32)), ..default()},
                texture: asset_server.load("wall.png"), transform: Transform::from_xyz(coords.0, coords.1, 0.0), ..default()
                })
                .insert(RigidBody::Fixed)
                .insert(Collider::cuboid(SPRITE_SIZE / 2.0, SPRITE_SIZE / 2.0))
                ;
            }
            else if playspace_matrix[i][j].gameobj_type == GameObjType::Player {
                commands.spawn().insert_bundle(SpriteBundle {
                    sprite: Sprite {custom_size: Option::Some(vec2(SPRITE_SIZE*1.2 as f32, SPRITE_SIZE*1.2 as f32)), ..default()},
                    texture: asset_server.load("Player.png"), transform: Transform::from_xyz(coords.0, coords.1, 0.0), ..default()
                })
                    .insert(Player)
                    .insert(RigidBody::Dynamic)
                    .insert(LockedAxes::ROTATION_LOCKED)
                    .insert(GravityScale(0.))
                    .insert(Ccd::enabled())
                    .insert(Velocity::zero())
                    .with_children(|cuboid_hitbox| {
                    cuboid_hitbox.spawn()
                        .insert(Collider::cuboid(SPRITE_SIZE / 2.75, SPRITE_SIZE / 2.15))
                        .insert(Transform::from_xyz(0., -5., 0.));
                });
            }
        }
    }
}

fn playspace_coords_to_world_coords(playspace_coords: (isize, isize)) -> (f32, f32, f32) {
    ((-WINDOW_WIDTH + SPRITE_SIZE)/2.0 + playspace_coords.0 as f32* SPRITE_SIZE,
    (WINDOW_HEIGHT - SPRITE_SIZE)/2.0 - playspace_coords.1 as f32 * SPRITE_SIZE,
    0.0 as f32)
}


// Add transform to the sprite