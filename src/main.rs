#![allow(unused)]

use bevy::{prelude::*, window::PrimaryWindow, sprite::collide_aabb::collide};
use components::{Velocity, Movable, Laser, FromPlayer, SpriteSize, Enemy, ExplosionToSpawn, Explosion, ExplosionTimer};
use enemy::EnemyPlugin;
use player::PlayerPlugin;

mod components;
mod player;
mod enemy;

// Assets Constans

const PLAYER_SPRITE: &str = "player_a_01.png";
const PLAYER_SIZE: (f32, f32) = (144., 75.);
const PLAYER_LASER_SPRITE: &str = "laser_a_01.png";
const PLAYER_LASER_SIZE: (f32, f32) = (9., 54.);

const ENEMY_SPRITE: &str = "enemy_a_01.png";
const ENEMY_SIZE: (f32, f32) = (144., 75.);
const ENEMY_LASER_SPRITE: &str = "laser_b_01.png";
const ENEMY_LASER_SIZE: (f32, f32) = (17., 55.);

const EXPLOSION_SHEET: &str = "explo_a_sheet.png";
const EXPLOSION_LEN: usize =  16;

const SPRITE_SCALE: f32 = 0.5;

// Game Constants
const TIME_STEP: f32 = 1. / 60.;
const BASE_SPEED: f32 = 500.;
const PLAYER_RESPAWN_DELAY: f64 = 2.;

// Resources
#[derive(Resource)]
pub struct WinSize {
	pub w: f32,
	pub h: f32,
}

#[derive(Resource)]
pub struct GameTexture {
    player: Handle<Image>,
    player_laser: Handle<Image>,
    enemy: Handle<Image>,
    enemy_laser: Handle<Image>,
    explosion: Handle<TextureAtlas>,
}

#[derive(Resource)]
struct PlayerState {
	on: bool,       // alive
	last_shot: f64, // -1 if not shot
}
impl Default for PlayerState {
	fn default() -> Self {
		Self {
			on: false,
			last_shot: -1.,
		}
	}
}

impl PlayerState {
	pub fn shot(&mut self, time: f64) {
		self.on = false;
		self.last_shot = time;
	}
	pub fn spawned(&mut self) {
		self.on = true;
		self.last_shot = -1.;
	}
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rust Invaders".into(),
                resolution: (598., 676.).into(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(PlayerPlugin)
        .add_plugins(EnemyPlugin)
        .add_systems(Startup, setup_system)
        .add_systems(Update, movable_system)
        .add_systems(Update, player_laser_hit_enemy_system)
        .add_systems(Update, explosion_to_spawn_system)
        .add_systems(Update, explosion_animation_system)
        .run();
}

fn setup_system(
    mut commands: Commands,
    assert_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    query: Query<&Window, With<PrimaryWindow>>,
) {
    // camera
    commands.spawn(Camera2dBundle::default());

    // capture window size
    let Ok(primary) = query.get_single() else {
        return;
    };
    let (win_w, win_h) = (primary.width(), primary.height());

    // position window (for tutorial)
	// window.set_position(IVec2::new(2780, 4900));

    // add winSize source
    let win_size = WinSize { w: win_w, h: win_h };
    commands.insert_resource(win_size);

    // create explosion texture atlas
    let texture_handle = assert_server.load(EXPLOSION_SHEET);
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64., 64.), 4, 4, None, None);
    let explosion = texture_atlases.add(texture_atlas);


    // Add Game Texture
    let game_texture = GameTexture {
        player: assert_server.load(PLAYER_SPRITE),
        player_laser: assert_server.load(PLAYER_LASER_SPRITE),
        enemy: assert_server.load(ENEMY_SPRITE),
        enemy_laser: assert_server.load(ENEMY_LASER_SPRITE),
        explosion,
    };

    commands.insert_resource(game_texture)
}

fn movable_system(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut query: Query<(Entity, &Velocity, &mut Transform, &Movable)> 
) {
    for (entity, velocity, mut transform, movable) in query.iter_mut() {
        let translation = &mut transform.translation;
        translation.x += velocity.x * TIME_STEP * BASE_SPEED;
        translation.y += velocity.y * TIME_STEP * BASE_SPEED;

        if movable.auto_despawn {
            // despawn when out of screen
            const MARGIN: f32 = 200.;
            if translation.y > win_size.h / 2. + MARGIN
                || translation.y < -win_size.h / 2. - MARGIN
                || translation.x > win_size.w / 2. + MARGIN
                || translation.x < -win_size.w / 2. - MARGIN
            {
                println!("->> despawn {entity:?}");
                commands.entity(entity).despawn();
            }
        }
    }
}

fn player_laser_hit_enemy_system(
    mut commands: Commands,
    laser_query: Query<(Entity, &Transform, &SpriteSize), (With<Laser>, With<FromPlayer>)>,
    enemy_query: Query<(Entity, &Transform, &SpriteSize), With<Enemy>>,
) {
    for (laser_entity, laser_tf, laser_size) in laser_query.iter() {
        let laser_scale = Vec2::from(laser_tf.scale.xy());

        for (enemy_entity, enemy_tf, enemy_size) in enemy_query.iter() {
            let enemy_scale = Vec2::from(enemy_tf.scale.xy());

            let collision = collide(
                laser_tf.translation,
                laser_size.0 * laser_scale,
                enemy_tf.translation,
                enemy_size.0 * enemy_scale,
            );

            if let Some(_) = collision {
                // remove enemy
                commands.entity(enemy_entity).despawn();

                // remove laser
                commands.entity(laser_entity).despawn();

                // spawn the explotions to spawn
                commands.spawn(ExplosionToSpawn(enemy_tf.translation.clone()));
            }
        }
    }
}

fn explosion_to_spawn_system(
    mut commands: Commands,
    game_textures: Res<GameTexture>,
    query: Query<(Entity, &ExplosionToSpawn)>,
) {
    for (explosion_spawn_entity, explosion_to_spawn) in query.iter() {
        // spawn explosion sprite
        commands.spawn(SpriteSheetBundle {
            texture_atlas: game_textures.explosion.clone(),
            transform: Transform {
                translation: explosion_to_spawn.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Explosion)
        .insert(ExplosionTimer::default());

        // despawn the explotionToSpawn
        commands.entity(explosion_spawn_entity).despawn();
    }
}

fn explosion_animation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ExplosionTimer, &mut TextureAtlasSprite), With<Explosion>>,
) {
    for (entity, mut timer, mut sprite) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            sprite.index += 1; // move to next sprite cell

            if sprite.index >= EXPLOSION_LEN {
                commands.entity(entity).despawn();
            }
        }
    }
}
