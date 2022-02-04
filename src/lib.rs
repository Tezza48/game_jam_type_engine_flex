use std::{time::{Duration, Instant}, collections::HashMap, cell::{Cell, RefCell}, borrow::Borrow};

use image::EncodableLayout;
use minifb::{Scale, ScaleMode, Window, WindowOptions};

pub fn start() {
    let mut next_entity = 0;
    let mut components = ComponentDatabase {
        start_time_instant: Instant::now(),
        last_frame_instant: Instant::now(),
        this_frame_instant: Instant::now(),
        delta_time: Duration::default(),
        total_time: Duration::default(),
        positions: HashMap::new(),
        sprites: HashMap::new(),
        render_targets: HashMap::new(),
        wobble_move: HashMap::new(),
    };

    let rt_entity = next_entity;
    next_entity += 1;
    components.render_targets.insert(rt_entity, RenderTarget);
    let (width, height) = (320, 180);
    components.sprites.insert(rt_entity, Sprite {
        data: vec![0; width * height],
        width: width as u32,
        height: height as u32,
        anchor_x: 0,
        anchor_y: 0,
    });

    let mut window = Window::new(
        "Engine",
        width,
        height,
        WindowOptions {
            scale: Scale::X1,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..Default::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.limit_update_rate(Some(Duration::from_millis(16)));

    let sprite_entity = next_entity;
    next_entity += 1;

    let img = image::open("assets/images/tezza_face.jpeg").unwrap();
    let img = img.flipv();
    let sprite_width = img.width();
    let sprite_height = img.height();

    let data = img.as_rgb8().unwrap().pixels().map(|pixel| {
        return (0xff as u32) << 24 | (pixel.0[0] as u32) << 16 | (pixel.0[1] as u32) << 8 | pixel.0[2] as u32;
    }).collect::<Vec<u32>>();

    components.positions.insert(sprite_entity, Position { x: width as i32, y: (height / 2) as i32});
    components.sprites.insert(sprite_entity, Sprite {
        data: data,
        width: sprite_width,
        height: sprite_height,
        anchor_x: sprite_width / 2,
        anchor_y: sprite_height / 2,
    });
    components.wobble_move.insert(sprite_entity, WobbleMove {
        amplitude: 160.0,
    });

    components.start_time_instant = std::time::Instant::now();
    components.last_frame_instant = std::time::Instant::now();

    let systems: Vec<fn(&mut ComponentDatabase)> = vec![
        sys_clear_render_target,
        sys_time_resources,
        sys_wobble_move,
        sys_draw_sprites,
    ];

    while window.is_open() {
        systems.iter().for_each(|f| {f(&mut components)});

        if let Some((rt_entity, _)) = components.render_targets.iter().next() {
            if let Some(render_target) = components.sprites.get(&rt_entity) {
            window
                .update_with_buffer(&render_target.data, width, height)
                .unwrap();
            }
        }
    }
}

fn sys_time_resources(components: &mut ComponentDatabase) {
    components.last_frame_instant = components.this_frame_instant;
    components.this_frame_instant = std::time::Instant::now();

    components.delta_time = components.this_frame_instant - components.last_frame_instant;
    components.total_time = components.this_frame_instant - components.start_time_instant;
}

struct ComponentDatabase {
    start_time_instant: Instant,
    last_frame_instant: Instant,
    this_frame_instant: Instant,
    delta_time: Duration,
    total_time: Duration,
    positions: HashMap<u32, Position>,
    sprites: HashMap<u32, Sprite>,
    render_targets: HashMap<u32, RenderTarget>,
    wobble_move: HashMap<u32, WobbleMove>,
}

struct Position {
    x: i32,
    y: i32,
}

struct Sprite {
    data: Vec<u32>,
    width: u32,
    height: u32,
    anchor_x: u32,
    anchor_y: u32,
}

struct RenderTarget;

struct WobbleMove {
    amplitude: f32,
}

fn sys_clear_render_target(components: &mut ComponentDatabase) {
    if let Some((rt_entity, _)) = components.render_targets.iter().next() {
        if let Some(sprite) = components.sprites.get_mut(&rt_entity) {
            sprite.data.fill(0);
        }
    }
}

fn sys_draw_sprites(components: &mut ComponentDatabase) {
    let render_target_entity = *if let Some((e, _)) = components.render_targets.iter().next() {e} else {return};
    // Take the render target.
    let mut target = if let Some(t) = components.sprites.remove(&render_target_entity) {t} else {return};


    for (entity, sprite) in components.sprites.iter() {
        let pos = if let Some(p) = components.positions.get(entity) {p} else {continue;};
        draw_sprite(&mut target, pos, sprite);
    }

    // Replace the render target
    components.sprites.insert(render_target_entity, target);
}

fn sys_wobble_move(components: &mut ComponentDatabase) {
    let rt_entity = *if let Some((e, _)) = components.render_targets.iter().next() {e} else {return;};
    let width = components.sprites[&rt_entity].width;

    let total_time = components.total_time.as_secs_f32();

    for (entity, wobble) in components.wobble_move.iter() {
        if let Some(pos) = components.positions.get_mut(entity) {
            pos.x = (width / 2) as i32 + (total_time.sin() * wobble.amplitude) as i32;
        }
    }
}

// fn sys_draw_sprites

fn draw_sprite(target: &mut Sprite, pos: &Position, sprite: &Sprite) {
    let anchored_pos_x = pos.x - sprite.anchor_x as i32;
    let anchored_pos_y = pos.y - sprite.anchor_y as i32;
    // calculate bounds we'll draw on the target,
    if anchored_pos_x + (sprite.width as i32) < 0
        || anchored_pos_y + (sprite.height as i32) < 0
        || anchored_pos_x > (target.width as i32)
        || anchored_pos_y > (target.height as i32)
    {
        return;
    }

    // TODO WT: Calculate the target and source bounds outside of the loop?

    for y in 0..sprite.height {
        for x in 0..sprite.width {
            let target_x = x as i32 + anchored_pos_x;
            let target_y = y as i32 + anchored_pos_y;

            if target_x >= 0
                && target_x < target.width as i32
                && target_y >= 0
                && target_y < target.height as i32
            {
                target.data[(target_y as u32 * target.width + target_x as u32) as usize] =
                    sprite.data[(y as u32 * sprite.width + x as u32) as usize];
            }
        }
    }
}
