#![feature(box_syntax)]
#![feature(type_ascription)]
#![feature(try_from)]
#![feature(specialization)]
#![feature(step_by)]
#![feature(slice_patterns)]
#![feature(zero_one)]

extern crate sdl2;
extern crate sdl2_image;
extern crate sdl2_ttf;
extern crate itertools;
extern crate rand;
extern crate chrono;

#[macro_use]
mod macros;
mod events;
mod view;
mod time;
mod gameobjects;
mod set;
mod graphics;
mod coalesce;
mod split_iterator;
mod fixed_size_iter;

use graphics::font_cache::FontCache;
use graphics::sprites::CopyRenderable;
use gameobjects::player::*;
use gameobjects::main_menu::main_menu;
use events::*;
use view::*;
use chrono::{UTC, Duration};

fn main() {
    let sdl = sdl2::init().unwrap();
    let sdl_ttf = sdl2_ttf::init().unwrap();
    let video = sdl.video().unwrap();

    let window = video
        .window("Test game", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut renderer = window.renderer()
        .accelerated()
        .build()
        .unwrap();

    let mut events = EventStream::new(sdl.event_pump().unwrap());
    let mut keys = Keys::default();

    let mut font_cache = FontCache::new(&sdl_ttf);

    let mut state = box main_menu(
        &mut renderer,
        &mut font_cache,
        box ShipViewBuilder
    ) as Box<View<_, _>>;
    let mut time = UTC::now();

    let target_ms_per_frame = Duration::milliseconds(1_000 / 60);

    loop {
        let now = UTC::now();
        let elapsed = now - time;
        let elapsed_ms = elapsed.num_milliseconds() as u32;

        let new_keys = events.pump(&keys);

        {
            let mut context =
                Context {
                    events: KeyEvents::new(
                        keys.clone(),
                        new_keys.clone(),
                    ),
                    renderer: &mut renderer,
                    font_cache: &mut font_cache,
                };

            if context.events.down.quit { break; }

            let next_state = match state.update(&mut context, elapsed_ms) {
                Action::Quit =>
                    break,
                Action::ChangeView(next) =>
                    Some(next.build_view(&mut context)),
                Action::Render(vec) => {
                    for (sprite, dest) in vec {
                        context.renderer.copy_renderable(
                            &sprite,
                            dest
                        );
                    }

                    None
                },
            };

            if let Some(n) = next_state {
                state = n;
            }
        }

        renderer.present();

        if elapsed < target_ms_per_frame {
            use std::thread;

            thread::sleep((target_ms_per_frame - elapsed).to_std().unwrap());
        }

        time = now;
        keys = new_keys;
    }
}
