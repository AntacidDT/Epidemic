/// Core game state — pure logic, no rendering dependencies.
#[derive(Debug)]
pub struct GameState {
    pub tick: u64,
    pub paused: bool,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            tick: 0,
            paused: false,
        }
    }

    pub fn advance(&mut self) {
        if !self.paused {
            self.tick += 1;
        }
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
