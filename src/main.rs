mod terminal;
mod task_graph;

use crossterm::QueueableCommand;
use terminal::renderer::Renderer;

#[tokio::main]
async fn main() {
    // for _ in 0..50 {
    //     println!("===============================================");
    // }
    let mut renderer = Renderer::new().unwrap();
    renderer.p_info();
    renderer.render_loop().unwrap();
}
