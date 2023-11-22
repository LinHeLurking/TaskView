mod terminal;
mod task_graph;

use terminal::Terminal;

fn main() {
    if let Some((w, h)) = Terminal::get_size() {
        println!("Width: {}\nHeight: {}", w, h);
    } else {
        println!("Unable to get term size :(")
    }
}
