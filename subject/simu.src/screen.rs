extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 128;
pub const MEM_SCREEN_BEGIN: usize = 0x10000;

pub struct Memory {
    pub m: Vec<u64>, 
}

impl Memory {
    pub fn new(size: usize) -> Self {
        Memory {
            m: vec![0; size], 
        }
    }
}
pub fn simulate_screen(m: Arc<Mutex<Memory>>, refresh: Arc<Mutex<bool>>) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Asm", (WIDTH * 2) as u32, (HEIGHT * 2) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::ARGB8888, WIDTH as u32, HEIGHT as u32)
        .unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut last_time = Instant::now();
    let mut tempscreen = vec![0u32; WIDTH * HEIGHT];

    let mut escape = false;

    while !escape {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => escape = true,
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => escape = true,
                _ => {}
            }
        }
        {
            let mem = m.lock().unwrap();
            for i in 0..(WIDTH * HEIGHT) {
                let mword = mem.m[(MEM_SCREEN_BEGIN >> 6) + (i >> 2)];
                let pixel = ((mword >> ((i & 3) << 4)) & 0xFFFF) as u32;

                let blue = pixel & ((1 << 5) - 1);
                let green = (pixel >> 5) & ((1 << 5) - 1);
                let red = pixel >> 10;
                tempscreen[i] = (red << (2 + 16)) + (green << (3 + 8)) + (blue << 3);
            }
        }
        texture
            .update(None, &tempscreen, WIDTH * 4)
            .expect("Failed to update texture");
        canvas.clear();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();
        let frame_duration = Duration::from_secs_f32(1.0 / 60.0);
        let elapsed = last_time.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
        last_time = Instant::now();
    }
    drop(texture);
    drop(canvas);
    sdl_context.quit();
}
