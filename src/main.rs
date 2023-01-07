use futures::io::Window;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, Palette};
use sdl2::rect::Point;
use sdl2::surface::Surface;
use sdl2::video::WindowSurfaceRef;
use std::time::Duration;

const PIXEL_OFF_COLOR: Color = Color::WHITE;
const PIXEL_ON_COLOR: Color = Color::BLACK;
const DISPLAY_WIDTH: u32 = 64;
const DISPLAY_HEIGHT: u32 = 32;
const DISPLAY_SCALE: u32 = 10;

pub fn main() -> Result<(), String> {
    let mut display_buffer: Vec<u8> = Vec::with_capacity(64 * 32);
    let mut memory: Vec<u8> = Vec::with_capacity(4096);
    let mut stack: Vec<u8> = Vec::new();

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window(
            "rust-sdl2 demo",
            DISPLAY_WIDTH * DISPLAY_SCALE,
            DISPLAY_HEIGHT * DISPLAY_SCALE,
        )
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

    // let mut canvas = window
    //     .into_canvas()
    //     .build()
    //     .expect("could not make a canvas");

    let mut event_pump = sdl_context.event_pump()?;

    let mut buf: Vec<u8> = vec![0; (DISPLAY_HEIGHT * DISPLAY_WIDTH) as usize];
    let mut surface = Surface::from_data(
        &mut buf,
        DISPLAY_WIDTH.into(),
        DISPLAY_HEIGHT.into(),
        DISPLAY_WIDTH.into(),
        sdl2::pixels::PixelFormatEnum::Index8,
    )?;
    let palette = Palette::with_colors(&[PIXEL_OFF_COLOR, PIXEL_ON_COLOR])?;
    surface.set_palette(&palette)?;

    'running: loop {
        // canvas.set_draw_color(PIXEL_OFF_COLOR);
        // canvas.clear();
        // i = (i + 1) % 600;
        // canvas.set_draw_color(PIXEL_ON_COLOR);
        // canvas.draw_point(Point::new(i, i))?;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        let mut window_surface = window.surface(&event_pump)?;
        let converted = surface.convert(&window_surface.pixel_format())?;
        converted.blit_scaled(None, &mut window_surface, None)?;
        window_surface.update_window()?;
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    Ok(())
}
