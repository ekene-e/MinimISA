extern crate sdl2;

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::process::exit;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use sdl2::event::Event;
use std::sync::Mutex;

mod memory;
mod processor;
mod screen;

use memory::Memory;
use processor::Processor;
use screen::simulate_screen;

fn usage() {
    eprintln!("Usage: simu [options] file.obj\nOptions: -d for debug, -s for step by step, -g for graphical screen");
    exit(1);
}

fn get_cmd_option(args: &[String], option: &str) -> Option<String> {
    let pos = args.iter().position(|s| s == option)?;
    if pos + 1 < args.len() {
        Some(args[pos + 1].clone())
    } else {
        None
    }
}

fn cmd_option_exists(args: &[String], option: &str) -> bool {
    args.iter().any(|s| s == option)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        usage();
    }

    let debug = cmd_option_exists(&args, "-d");
    let step_by_step = cmd_option_exists(&args, "-s");
    let graphical_output = cmd_option_exists(&args, "-g");

    let filename = args.last().expect("No filename provided").clone();

    if let Err(_) = File::open(&filename) {
        eprintln!("Can't access obj file");
        usage();
    }

    let memory = Arc::new(Mutex::new(Memory::new()));
    let processor = Processor::new(Arc::clone(&memory));

    memory.lock().unwrap().fill_with_obj_file(&filename);

    let refresh = Arc::new(AtomicBool::new(true));
    let quit_signal = Arc::new(AtomicBool::new(false));

    let screen_thread = if graphical_output {
        let mem_clone = Arc::clone(&memory);
        let refresh_clone = Arc::clone(&refresh);
        let quit_signal_clone = Arc::clone(&quit_signal);

        Some(thread::spawn(move || {
            simulate_screen(&mem_clone, &refresh_clone, &quit_signal_clone);
        }))
    } else {
        None
    };

    loop {
        processor.von_neumann_step(debug);

        if step_by_step {
            let _ = std::io::stdin().read_line(&mut String::new());
        }
    }

    if let Some(screen_thread) = screen_thread {
        quit_signal.store(true, Ordering::SeqCst);  
        screen_thread.join().unwrap();  
    }
}
