mod chip8;

extern crate sdl2;

use chip8::Chip8;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use core::panic;
use std::fs::File;
use std::io::Read;
use std::time::Duration;
//use sdl2::AudioSubsystem;

use sdl2::render::Canvas;
use sdl2::video::Window;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 320;

const SCALE: u32 = 10;

fn set_pixel(canvas: &mut Canvas<Window>, x: i32, y: i32) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    let rect = sdl2::rect::Rect::new(x, y, SCALE, SCALE);
    canvas.fill_rect(rect)?;
    Ok(())
}

fn clear_screen(canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
}

fn draw(canvas: &mut Canvas<Window>, screen: &[u8;  64 * 32]) -> Result<(), String>{
    clear_screen(canvas);
    for (i, pixel) in screen.iter().enumerate() {
        if *pixel == 1 {
            let x = (i % 64) as i32 * SCALE as i32;
            let y = (i / 64) as i32 * SCALE as i32;
            set_pixel(canvas, x, y)?;
        }
    }
    Ok(())
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;

    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .target_texture()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    let mut emu = Chip8::new();
    let mut file = File::open("roms/pong.ch8").unwrap();
    //let mut file = File::open("roms/5-quirks.ch8").unwrap();
    let mut rom = Vec::new();
    file.read_to_end(&mut rom).unwrap();

    emu.load_rom(rom);

    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode, ..} => emu.set_key(keycode.unwrap(), 1),
                Event::KeyUp { keycode, ..} => emu.set_key(keycode.unwrap(), 0),
                _ => {}
            }
        }

        canvas.clear();
        emu.clock();
        draw(&mut canvas, &emu.screen)?;

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    Ok(())
}