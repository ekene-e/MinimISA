use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// Breakpoint manager structure to manage breakpoints
pub struct BreakpointManager {
    breakpoints: Arc<Mutex<HashSet<u64>>>,  
}

impl BreakpointManager {
    pub fn new() -> Self {
        BreakpointManager {
            breakpoints: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn add(&self, address: u64) {
        let mut breaks = self.breakpoints.lock().unwrap();
        breaks.insert(address);
    }

    pub fn remove(&self, address: u64) -> Result<(), String> {
        let mut breaks = self.breakpoints.lock().unwrap();
        if breaks.remove(&address) {
            Ok(())
        } else {
            Err(format!("Breakpoint not found at address: 0x{:x}", address))
        }
    }

    pub fn has(&self, address: u64) -> bool {
        let breaks = self.breakpoints.lock().unwrap();
        breaks.contains(&address)
    }

    pub fn show(&self) {
        let breaks = self.breakpoints.lock().unwrap();
        if breaks.is_empty() {
            println!("No breakpoints set.");
        } else {
            println!("Breakpoints:");
            for &bp in breaks.iter() {
                println!(" - 0x{:x}", bp);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breakpoint_management() {
        let manager = BreakpointManager::new();

        manager.add(0x1000);
        manager.add(0x2000);

        assert!(manager.has(0x1000));
        assert!(manager.has(0x2000));
        assert!(!manager.has(0x3000));

        manager.remove(0x1000).unwrap();
        assert!(!manager.has(0x1000));

        assert!(manager.remove(0x3000).is_err());

        manager.show();
    }
}
