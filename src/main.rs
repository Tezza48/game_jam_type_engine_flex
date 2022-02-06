mod entity;

use std::time::{Duration, Instant};

use entity::{Component, Entity, FindEntityWithComponent, FindEntityWithComponentMut};
use minifb::{Scale, ScaleMode, Window, WindowOptions};

struct TimeResources {
    start_time_instant: Instant,
    last_frame_instant: Instant,
    this_frame_instant: Instant,
    delta_time: Duration,
    total_time: Duration,
}

impl Component for TimeResources {}

struct Position {
    x: i32,
    y: i32,
}

impl Component for Position {}

struct Sprite {
    data: Vec<u32>,
    width: u32,
    height: u32,
    anchor_x: u32,
    anchor_y: u32,
}

impl Component for Sprite {}

struct RenderTarget;

impl Component for RenderTarget {}

struct WobbleMove {
    amplitude: f32,
}

impl Component for WobbleMove {}

pub fn start() {
    let mut resources = Entity::new();
    resources.add_component(TimeResources {
        start_time_instant: Instant::now(),
        last_frame_instant: Instant::now(),
        this_frame_instant: Instant::now(),
        delta_time: Duration::default(),
        total_time: Duration::default(),
    });

    let (width, height) = (320, 180);
    let mut entities = Vec::new();

    entities.push({
        let mut rt_entity = Entity::new();
        rt_entity.add_component(RenderTarget);
        rt_entity.add_component(Sprite {
            data: vec![0; width * height],
            width: width as u32,
            height: height as u32,
            anchor_x: 0,
            anchor_y: 0,
        });

        rt_entity
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

    entities.push({
        let mut sprite_entity = Entity::new();

        let img = image::open("assets/images/tezza_face.jpeg").unwrap();
        let img = img.flipv();
        let sprite_width = img.width();
        let sprite_height = img.height();

        let data = img
            .as_rgb8()
            .unwrap()
            .pixels()
            .map(|pixel| {
                return (0xff as u32) << 24
                    | (pixel.0[0] as u32) << 16
                    | (pixel.0[1] as u32) << 8
                    | pixel.0[2] as u32;
            })
            .collect::<Vec<u32>>();

        sprite_entity.add_component(Position {
            x: width as i32,
            y: (height / 2) as i32,
        });
        sprite_entity.add_component(Sprite {
            data: data,
            width: sprite_width,
            height: sprite_height,
            anchor_x: sprite_width / 2,
            anchor_y: sprite_height / 2,
        });
        sprite_entity.add_component(WobbleMove { amplitude: 160.0 });

        sprite_entity
    });

    {
        let time_resources = resources.get_component_mut::<TimeResources>().unwrap();

        time_resources.start_time_instant = std::time::Instant::now();
        time_resources.last_frame_instant = std::time::Instant::now();
    }

    let systems: Vec<fn(&mut Entity, &mut Vec<Entity>)> = vec![
        sys_clear_render_target,
        sys_time_resources,
        sys_wobble_move,
        sys_draw_sprites,
    ];

    while window.is_open() {
        systems
            .iter()
            .for_each(|f| f(&mut resources, &mut entities));

        if let Some(entity) = entities.iter()
        .find(|e| e.has_component::<RenderTarget>())
         {
            let sprite: &Sprite = entity.get_component().unwrap();

            window
                .update_with_buffer(&sprite.data, sprite.width as usize, sprite.height as usize)
                .unwrap();
        } else {
            panic!("For some reason there's no render target entity")
        };
    }
}

fn sys_time_resources(resources: &mut Entity, _: &mut Vec<Entity>) {
    let time_resources: &mut TimeResources = resources.get_component_mut().unwrap();

    time_resources.last_frame_instant = time_resources.this_frame_instant;
    time_resources.this_frame_instant = std::time::Instant::now();

    time_resources.delta_time =
        time_resources.this_frame_instant - time_resources.last_frame_instant;
    time_resources.total_time =
        time_resources.this_frame_instant - time_resources.start_time_instant;
}

fn sys_clear_render_target(_: &mut Entity, entities: &mut Vec<Entity>) {
    if let Some(render_target_entity) = entities
        .iter_mut()
        .find_entity_with_component_mut::<RenderTarget>()
    {
        if let Some(sprite) = render_target_entity.get_component_mut::<Sprite>() {
            sprite.data.fill(0xff000000);
        } else {
            panic!("RenderTarget entity didn't have a sprite component");
        }
    } else {
        panic!("No RenderTarget entity exists");
    };
}

fn sys_draw_sprites(_: &mut Entity, entities: &mut Vec<Entity>) {
    // TODO WT: Easier way of taking and replacing a component.
    // TODO WT: Maybe something which takes the component and replaces it once dropped.
    let mut target = if let Some(e) = entities
        .iter_mut()
        .find_entity_with_component_mut::<RenderTarget>()
    {
        e.remove_component::<Sprite>()
    } else {
        return;
    };

    for entity in entities.iter() {
        if let (Some(sprite), Some(pos)) = (
            entity.get_component::<Sprite>(),
            entity.get_component::<Position>(),
        ) {
            draw_sprite(&mut target, pos, sprite);
        }
    }

    if let Some(e) = entities
        .iter_mut()
        .find_entity_with_component_mut::<RenderTarget>()
    {
        e.add_component(target);
    }
}

fn sys_wobble_move(resources: &mut Entity, entities: &mut Vec<Entity>) {
    let width = if let Some(e) = entities.iter().find_entity_with_component::<RenderTarget>() {
        e.get_component::<Sprite>()
            .unwrap()
            .width
    } else {
        return;
    };

    let total_time = resources
        .get_component::<TimeResources>()
        .unwrap()
        .total_time
        .as_secs_f32();

    for entity in entities.iter_mut() {
        let amplitude = if let Some(wobble) = entity.get_component::<WobbleMove>() {
            wobble.amplitude
        } else {
            continue;
        };

        if let Some(pos) = entity.get_component_mut::<Position>() {
            pos.x = (width / 2) as i32 + (total_time.sin() * amplitude) as i32;
        }
    }
}

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

fn main() {
    start();
}
