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
static WINDOW_WIDTH: f32 = 800 as f32;
static WINDOW_HEIGHT: f32 = 600 as f32;
static SPRITE_SIZE: f32 = 100.0;  // SIZE MUST ALWAYS BE A FACTOR OF WINDOW_WIDTH AND WINDOW_HEIGHT!!
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
        .add_plugin(RapierDebugRenderPlugin::default())
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
struct PlayspaceMatrix(Vec<Vec<GameObject>>);

impl PlayspaceMatrix {
    fn instantiate_gameobject_at_coord (&mut self, coord : (usize, usize), gameobj_type: GameObjType) {
        match gameobj_type {
            GameObjType::Wall => {self.0[coord.0][coord.1] = GameObject {gameobj_type: GameObjType::Wall, playspace_position: coord, symbol: "W".to_string()}}
            GameObjType::Player => {self.0[coord.0][coord.1] = GameObject {gameobj_type: GameObjType::Player, playspace_position: coord, symbol: "P".to_string()}}
            Empty => {self.0[coord.0][coord.1] = GameObject {gameobj_type: Empty, playspace_position: coord, symbol: "0".to_string()}}
        };
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

#[derive(Component)]
struct Player;


// This is where we 'set' the game data!! (we impl Default for all resources)
impl FromWorld for GameState {
    fn from_world(world: &mut World) -> Self {
        let mut game_state = Self {
            player_spawn_pos: (0, 0),
            playspace_matrix: PlayspaceMatrix::default()
        };

        // Walls on the border
        // top and bottom
        let playspace_width_usize = PLAYSPACE_WIDTH as usize;
        let playspace_height_usize = PLAYSPACE_HEIGHT as usize;
        let mut coords_border_wall : Vec<(usize, usize)> = Vec::new();
        for i in 0..playspace_width_usize as usize {
            coords_border_wall.push((0, i));
            coords_border_wall.push((playspace_height_usize-1, i));
            // game_state.playspace_matrix.0[0][i] = "W".to_string();
            // game_state.playspace_matrix.0[playspace_height_usize -1][i] = "W".to_string();
        }
        // left and right
        for i in 1..playspace_height_usize as usize{
            coords_border_wall.push((i, 0));
            coords_border_wall.push((i, playspace_width_usize-1));
            // game_state.playspace_matrix.0[i][0] = "W".to_string();
            // game_state.playspace_matrix.0[i][playspace_width_usize -1] = "W".to_string();
        }

        for current_cord in coords_border_wall.clone() {
            // game_state.playspace_matrix.0[current_cord.0][current_cord.1] = "W".to_string()
            game_state.playspace_matrix.instantiate_gameobject_at_coord((current_cord.0, current_cord.1), GameObjType::Wall);
        }
        // Player inside
        let mut rng = rand::thread_rng();
        let player_x = rng.gen_range(1..PLAYSPACE_WIDTH-1) as usize;    // PLAYSPACE_WIDTH-1 because i dont want the player to spawn on a wall
        let player_y = rng.gen_range(1..PLAYSPACE_HEIGHT-1) as usize;
        // game_state.playspace_matrix.0[player_y][player_x] = "P".to_string();
        game_state.playspace_matrix.instantiate_gameobject_at_coord((player_y, player_x), GameObjType::Player);


        // Procedurally randomly generated inner walls
        // let root_coords_for_generation = coords_border_wall.clone();
        // let random_border_wall_coord = root_coords_for_generation.choose(&mut rand::thread_rng()).unwrap();
        // game_state.playspace_matrix.0[random_border_wall_coord.0][random_border_wall_coord.1] = "0".to_string();

        game_state
    }
}

// impl PlayspaceMatrix {
//     fn instantiate_gameobject(mut self, coords: (x, y)) {
//
//     }
// }

#[derive(Clone)]
struct GameObject {
    gameobj_type: GameObjType,
    playspace_position: (usize, usize),
    symbol: String,
}

impl Default for GameObject {
    fn default() -> Self {
        Self {gameobj_type: GameObjType::Empty, playspace_position: (0,0), symbol: "X".to_string()}
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
                // println!("{} {}", i, j);
                // println!("{:?}", coords);
                commands.spawn().insert_bundle(SpriteBundle {
                sprite: Sprite {custom_size: Option::Some(vec2(SPRITE_SIZE as f32, SPRITE_SIZE as f32)), ..default()},
                texture: asset_server.load("wall.png"), transform: Transform::from_xyz(coords.0, coords.1, 0.0), ..default()
                })
                .insert(RigidBody::Fixed)
                // .insert(CollisionShape::Cuboid {half_extends: Vec3::new(SPRITE_SIZE / 2., SPRITE_SIZE / 2., 10.), border_radius: None })
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
                    // .insert(CollisionShape::Cuboid {half_extends: Vec3::new(SPRITE_SIZE / 2., SPRITE_SIZE / 2., 10.), border_radius: None })
                    .insert(Collider::cuboid(SPRITE_SIZE / 2.1, SPRITE_SIZE / 2.1))
                    ;
                ;
            }
        }
    }
    // commands.spawn().insert_bundle(SpriteBundle {
    //     // sprite: Sprite {custom_size: Option::Some(Vec2::from(get_sprite_size("assets/wall.png"))), ..default()},
    //     sprite: Sprite {custom_size: Option::Some(Vec2::from((SPRITE_SIZE, SPRITE_SIZE))), ..default()},
    //     texture: asset_server.load("wall.png"), transform: Transform::from_xyz((-WINDOW_WIDTH + SPRITE_SIZE)/2.0, (WINDOW_HEIGHT - SPRITE_SIZE)/2.0, 0.0), ..default()
    //
    // });

}

fn playspace_coords_to_world_coords(playspace_coords: (isize, isize)) -> (f32, f32, f32) {
    // ((playspace_coords.0 as f32 * SPRITE_SIZE - WINDOW_WIDTH as f32) / 2.0,
    //  (playspace_coords.1 as f32 * SPRITE_SIZE - WINDOW_HEIGHT as f32) / 2.0,1
    //  0.0)

    let coords = ((-WINDOW_WIDTH + SPRITE_SIZE)/2.0 + playspace_coords.0 as f32* SPRITE_SIZE,
                      (WINDOW_HEIGHT - SPRITE_SIZE)/2.0 - playspace_coords.1 as f32 * SPRITE_SIZE,
                      0.0 as f32);

    coords
}


// Add transform to the sprite