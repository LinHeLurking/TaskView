mod renderer;

use term_size::dimensions;


pub struct Terminal {}

impl Terminal {
    pub fn get_size() -> Option<(usize, usize)> {
        dimensions()
    }
}
