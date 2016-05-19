#![feature(fnbox)]
#![feature(try_from)]
#![feature(specialization)]
#![feature(step_by)]
#![feature(slice_patterns)]
#![feature(zero_one)]

extern crate sdl2;
extern crate sdl2_image;
extern crate sdl2_ttf;
extern crate itertools;

#[macro_use]
mod macros;
mod events;
mod view;
mod time;
mod gameobjects;
mod set;
mod graphics;
mod lazy;
mod coelesce;

use graphics::font_cache::FontCache;
use gameobjects::player::*;
use gameobjects::main_menu::main_menu;
use events::*;
use view::*;

fn main() {
    let sdl = sdl2::init().unwrap();
    let sdl_ttf = sdl2_ttf::init().unwrap();
    let video = sdl.video().unwrap();
    let mut timer = sdl.timer().unwrap();

    let window = video
        .window("Test game", 800, 600)
        .position_centered()
        .opengl()
        .resizable()
        .build()
        .unwrap();

    let mut renderer = window.renderer()
        .accelerated()
        .build()
        .unwrap();

    let mut events = EventStream::new(sdl.event_pump().unwrap());
    let mut keys = Keys::default();

    let mut font_cache = FontCache::new(&sdl_ttf);

    let ship_state = Box::new(ShipView::new(&mut renderer));
    let mut state = Box::new(
        main_menu(
            &mut renderer,
            &mut font_cache,
            ship_state
        )
    ) as Box<View<_>>;
    let mut time = timer.ticks();

    let target_ms_per_frame = 1_000 / 60;

    loop {
        let now = timer.ticks();
        let elapsed = now - time;

        let new_keys = events.pump(&keys);

        {
            let context =
                Context {
                    time: GameTime {
                        elapsed: elapsed,
                        total: now,
                    },
                    events: KeyEvents::new(
                        keys.clone(),
                        new_keys.clone(),
                    ),
                    renderer: &mut renderer,
                };

            if context.events.down.quit { break; }

            match state.render(context) {
                Some(Action::Quit) => break,
                Some(Action::ChangeView(next)) => state = next,
                None => { }
            }
        }

        renderer.present();

        time = now;
        keys = new_keys;

        if elapsed < target_ms_per_frame {
            timer.delay(target_ms_per_frame - elapsed);
        }
    }
}
