extern crate ncurses;

use crate::cpu::CPU;
use crate::memory::Memory;
use ncurses::*;
use std::fmt;
use std::sync::{Arc, Mutex};

// Ncurses window panels
pub struct Debugger {
    wcode: WINDOW,
    wreg: WINDOW,
    wmem: WINDOW,
    wframe: WINDOW,
    wcli: WINDOW,

    cpu: Arc<Mutex<CPU>>,
    memory: Arc<Mutex<Memory>>,
    state: DebuggerState,
}

#[derive(Debug, Clone, Copy)]
pub enum DebuggerState {
    Idle,   // Program is ready to run
    Break,  // Program has reached breakpoint
    Halt,   // Program has reached end or infinite loop
}

#[derive(Debug, Clone, Copy)]
pub enum DebuggerColor {
    Black = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    White = 7,

    Command = DebuggerColor::Cyan as isize,
    Error = DebuggerColor::Red as isize,
    Idle = DebuggerColor::Yellow as isize,
    Break = DebuggerColor::Cyan as isize,
    Halt = DebuggerColor::Green as isize,

    Arithm = DebuggerColor::White as isize,
    Test = DebuggerColor::White as isize,
    Let = DebuggerColor::Green as isize,
    Jump = DebuggerColor::Cyan as isize,
    Memory = DebuggerColor::Red as isize,
    Control = DebuggerColor::Magenta as isize,
}

impl Debugger {
    /// Create and initialize the debugger interface
    pub fn new(cpu: Arc<Mutex<CPU>>, memory: Arc<Mutex<Memory>>) -> Debugger {
        initscr();
        start_color();
        use_default_colors();
        Debugger::init_colors();

        Debugger {
            wcode: newwin(10, 50, 0, 0),
            wreg: newwin(10, 30, 0, 50),
            wmem: newwin(10, 30, 10, 0),
            wframe: newwin(10, 30, 10, 30),
            wcli: newwin(5, 80, 20, 0),

            cpu,
            memory,
            state: DebuggerState::Idle,
        }
    }

    /// Initialize color pairs
    fn init_colors() {
        init_pair(DebuggerColor::Black as i16, COLOR_BLACK, -1);
        init_pair(DebuggerColor::Red as i16, COLOR_RED, -1);
        init_pair(DebuggerColor::Green as i16, COLOR_GREEN, -1);
        init_pair(DebuggerColor::Yellow as i16, COLOR_YELLOW, -1);
        init_pair(DebuggerColor::Blue as i16, COLOR_BLUE, -1);
        init_pair(DebuggerColor::Magenta as i16, COLOR_MAGENTA, -1);
        init_pair(DebuggerColor::Cyan as i16, COLOR_CYAN, -1);
        init_pair(DebuggerColor::White as i16, COLOR_WHITE, -1);
    }

    /// Run the debugger (main loop)
    pub fn run(&mut self, filename: Option<&str>) {
        self.draw_interface();
        loop {
            match self.state {
                DebuggerState::Idle => {
                    // Process commands
                    let cmd = self.prompt();
                    self.handle_command(cmd);
                }
                DebuggerState::Break => {
                    self.log("Breakpoint reached.");
                    self.state = DebuggerState::Idle;
                }
                DebuggerState::Halt => {
                    self.log("Program halted.");
                    break;
                }
            }
        }
        endwin();  // End ncurses mode
    }

    /// Draw the interface panels
    fn draw_interface(&self) {
        // Draw the code, register, and memory panels
        self.code_panel();
        self.memory_panel();
        self.reg_panel();
        wrefresh(self.wcli);
    }

    /// Refresh the code panel, showing disassembled code
    fn code_panel(&self) {
        // Assuming there's a disassemble function available in CPU or Memory
        let code_listing = self.cpu.lock().unwrap().disassemble();
        mvwprintw(self.wcode, 1, 1, &code_listing);
        wrefresh(self.wcode);
    }

    /// Refresh the memory panel
    fn memory_panel(&self) {
        let mem_dump = self.memory.lock().unwrap().dump();
        mvwprintw(self.wmem, 1, 1, &mem_dump);
        wrefresh(self.wmem);
    }

    /// Refresh the register panel
    fn reg_panel(&self) {
        let reg_state = self.cpu.lock().unwrap().dump_registers();
        mvwprintw(self.wreg, 1, 1, &reg_state);
        wrefresh(self.wreg);
    }

    /// Move to a different section of memory
    fn memory_move(&self, address: u64) {
        self.memory.lock().unwrap().move_to_address(address);
        self.memory_panel();  // Refresh the memory panel
    }

    /// Prompt the user for input
    fn prompt(&self) -> String {
        let mut input = String::new();
        mvwgetstr(self.wcli, 1, 1, &mut input);
        input
    }

    /// Handle user commands
    fn handle_command(&mut self, cmd: String) {
        match cmd.as_str() {
            "run" => {
                self.state = DebuggerState::Idle;
            }
            "step" => {
                self.cpu.lock().unwrap().step();
                self.reg_panel();
            }
            "break" => {
                self.state = DebuggerState::Break;
            }
            "exit" => {
                self.state = DebuggerState::Halt;
            }
            _ => {
                self.log_error("Unknown command.");
            }
        }
    }

    /// Log messages to the console
    fn log(&self, message: &str) {
        wattron(self.wcli, COLOR_PAIR(DebuggerColor::Command as i16));
        mvwprintw(self.wcli, 1, 1, message);
        wattroff(self.wcli, COLOR_PAIR(DebuggerColor::Command as i16));
        wrefresh(self.wcli);
    }

    /// Log error messages
    fn log_error(&self, message: &str) {
        wattron(self.wcli, COLOR_PAIR(DebuggerColor::Error as i16));
        mvwprintw(self.wcli, 1, 1, &format!("error: {}", message));
        wattroff(self.wcli, COLOR_PAIR(DebuggerColor::Error as i16));
        wrefresh(self.wcli);
    }
}

