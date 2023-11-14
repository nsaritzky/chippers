use sdl2::{pixels::PixelFormat, surface::Surface, video::Window, EventPump};

use crate::{Emulator, DISPLAY_HEIGHT, DISPLAY_WIDTH};

pub fn render(
    window: &Window,
    event_pump: &EventPump,
    emulator: &Emulator,
    pixel_format: &PixelFormat,
) -> Result<(), String> {
    let mut window_surface = window.surface(event_pump)?;

    let mut expanded_buf = emulator.display_buffer.borrow_mut();

    let surface = Surface::from_data(
        &mut expanded_buf,
        DISPLAY_WIDTH,
        DISPLAY_HEIGHT,
        DISPLAY_WIDTH / 8,
        sdl2::pixels::PixelFormatEnum::Index1MSB,
    )?;

    let converted = surface.convert(pixel_format)?;
    converted.blit_scaled(None, &mut window_surface, None)?;
    window_surface.finish()?;
    Ok(())
}
