extern crate sdl2;

use sdl2::event::Event;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::time::Duration;

type Callback = Box<dyn Fn(&[u8], &mut dyn std::any::Any) + Send + 'static>;

pub struct Graphical {
    width: usize,
    height: usize,
    vram: Arc<Mutex<Vec<u8>>>,  
    scale: i32,
    callback: Option<Callback>,
    funcarg: Arc<Mutex<dyn std::any::Any + Send>>,
    stop_signal: Arc<(Mutex<bool>, Condvar)>, 
}

impl Graphical {
    pub fn new(
        width: usize,
        height: usize,
        vram: Vec<u8>,
        callback: Option<Callback>,
        funcarg: Arc<Mutex<dyn std::any::Any + Send>>,
        scale: i32,
    ) -> Self {
        Graphical {
            width,
            height,
            vram: Arc::new(Mutex::new(vram)),
            scale,
            callback,
            funcarg,
            stop_signal: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }

    /// Start the SDL thread for the screen
    pub fn start(&self) -> Result<(), String> {
        let vram = Arc::clone(&self.vram);
        let funcarg = Arc::clone(&self.funcarg);
        let callback = self.callback.as_ref().map(|cb| Arc::new(Mutex::new(cb)));
        let stop_signal = Arc::clone(&self.stop_signal);

        let (width, height, scale) = (self.width, self.height, self.scale);

        thread::spawn(move || {
            let sdl_context = sdl2::init().unwrap();
            let video_subsystem = sdl_context.video().unwrap();

            let window = video_subsystem
                .window(
                    "Graphical Window",
                    (width * scale as usize) as u32,
                    (height * scale as usize) as u32,
                )
                .position_centered()
                .build()
                .unwrap();

            let mut canvas = window.into_canvas().present_vsync().build().unwrap();
            let texture_creator = canvas.texture_creator();
            let mut texture = texture_creator
                .create_texture_streaming(PixelFormatEnum::RGB565, width as u32, height as u32)
                .unwrap();

            let mut event_pump = sdl_context.event_pump().unwrap();

            // Keep running until a stop signal is received
            let (lock, cvar) = &*stop_signal;
            'running: loop {
                // Check for stop signal
                if *lock.lock().unwrap() {
                    break 'running;
                }

                // Poll for SDL events
                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. } => break 'running,
                        _ => {}
                    }
                }

                // Call the callback function at 60 Hz
                if let Some(cb) = &callback {
                    let keyboard_state = event_pump.keyboard_state().scancodes().collect::<Vec<_>>();
                    let mut funcarg_locked = funcarg.lock().unwrap();
                    cb.lock().unwrap()(&keyboard_state, &mut *funcarg_locked);
                }

                // Lock the video memory (vram) and update the texture with it
                let vram_locked = vram.lock().unwrap();
                texture
                    .update(None, &vram_locked, (width * 2) as usize)
                    .expect("Failed to update texture");

                // Render the texture to the screen
                canvas.clear();
                canvas
                    .copy(&texture, None, Some(Rect::new(0, 0, (width * scale) as i32, (height * scale) as i32)))
                    .unwrap();
                canvas.present();

                // Sleep to maintain ~60Hz
                thread::sleep(Duration::from_millis(16));
            }

            // Clean up when the thread stops
            cvar.notify_all();
        });

        Ok(())
    }

    /// Send a refresh signal to the SDL thread (refreshes screen)
    pub fn refresh(&self) {
        // In this case, the event loop already handles refreshing
    }

    /// Stop regular update, to be used when the program ends
    pub fn freeze(&self) {
        // In this case, the event loop already handles refreshing
    }

    /// Wait for the SDL thread to stop and clean up
    pub fn wait(&self) {
        let (lock, cvar) = &*self.stop_signal;
        let mut stopped = lock.lock().unwrap();
        while !*stopped {
            stopped = cvar.wait(stopped).unwrap();
        }
    }

    /// Stop the SDL thread, send a quit event, and clean up
    pub fn stop(&self) {
        // Send the stop signal to the SDL thread
        let (lock, cvar) = &*self.stop_signal;
        let mut stop_flag = lock.lock().unwrap();
        *stop_flag = true;

        // Wait for the thread to stop and clean up resources
        cvar.notify_all();
    }
}
