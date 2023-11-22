use std::sync::{Arc, Mutex};
use crate::task_graph::task_dag::TaskDag;
use crate::terminal::renderer::RenderErrorCategory::MutexPoison;

type _C = char;

pub struct Renderer {
    on_scree_buf: Vec<_C>,
    pre_render_buf: Vec<_C>,
    info_collect_buf: Vec<_C>,
    w: usize,
    h: usize,
    task_graph: Arc<Mutex<TaskDag>>,
    new_layout: bool,
    finished: bool,
    layout: Layout,
}

struct Layout {}

pub enum RenderErrorCategory {
    MutexPoison,
}

pub struct RenderError {
    category: RenderErrorCategory,
    detail: String,
}

impl Renderer {
    fn render_single_frame(&mut self) -> Result<(), RenderError> {
        if self.new_layout { self.detect_layout()?; }
        self.collect_info()?;
        self.fill_pre_render()?;
        self.commit_to_screen()?;
        self.new_layout = false;
        Ok(())
    }

    pub async fn update_tasks(&mut self, tasks: TaskDag) -> Result<(), RenderError> {
        self.new_layout = true;
        let mut cur_tasks = self.task_graph.lock()
            .map_err(|e| RenderError {
                category: MutexPoison,
                detail: e.to_string(),
            })?;
        *cur_tasks = tasks;
        Ok(())
    }

    fn collect_info(&mut self) -> Result<(), RenderError> {
        Ok(())
    }

    fn fill_pre_render(&mut self) -> Result<(), RenderError> {
        Ok(())
    }

    fn commit_to_screen(&mut self) -> Result<(), RenderError> {
        Ok(())
    }

    fn detect_layout(&mut self) -> Result<(), RenderError> {
        Ok(())
    }

    fn detect_resize(&mut self) -> Result<(), RenderError> {
        Ok(())
    }

    pub async fn render_loop(&mut self) -> Result<(), RenderError> {
        self.detect_resize()?;
        self.detect_layout()?;
        let mut cnt = 0u64;
        while self.finished {
            self.render_single_frame()?;
            cnt += 1;
            if cnt == 20 {
                self.detect_resize()?;
                cnt = 0;
            }
        }
        Ok(())
    }
}