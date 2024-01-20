use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use rand::{thread_rng, Rng};

use crate::{GameTexture, SPRITE_SCALE, WinSize, components::{Enemy, SpriteSize}, ENEMY_SIZE};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, enemy_spawn_system.run_if(on_timer(Duration::from_secs_f32(1.))));
    }
}

fn enemy_spawn_system(
    mut commands: Commands,
    game_textures: Res<GameTexture>,
    win_size: Res<WinSize>,
) {
    // compute x/y
    let mut rng = thread_rng();
    let w_span = win_size.w / 2. - 100.;
    let h_span = win_size.h / 2. - 100.;
    let x = rng.gen_range(-w_span..w_span);
    let y = rng.gen_range(-h_span..h_span);

    commands.spawn(SpriteBundle{
        texture: game_textures.enemy.clone(),
        transform: Transform {
            translation: Vec3::new(x, y, 10.),
            scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.),
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(Enemy)
    .insert(SpriteSize::from(ENEMY_SIZE));
}