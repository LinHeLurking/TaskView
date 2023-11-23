use std::cmp::{max, min};
use std::io;
use std::sync::{Arc, Mutex, MutexGuard};
use term_size::dimensions;
use crossterm::{cursor, QueueableCommand, style::{Print}};
use std::io::{Write, stdout};
use crossterm::cursor::{MoveTo, MoveToColumn};
use crossterm::terminal::{ScrollDown, ScrollUp};
use crate::task_graph::task_dag::TaskDag;
use crate::terminal::renderer::RenderErrorCategory::{EMutex, ETerminal};

type _C = char;

pub struct Renderer {
    on_scree_buf: Vec<_C>,
    pre_render_buf: Vec<_C>,
    vw: usize,
    vh: usize,
    pw: usize,
    ph: usize,
    x: usize,
    y: usize,
    task_graph: Arc<Mutex<TaskDag>>,
    new_layout: Arc<Mutex<bool>>,
    finished: Arc<Mutex<bool>>,
    layout: Layout,
}

impl Renderer {
    pub fn new() -> Result<Renderer, RenderError> {
        let r = Renderer {
            on_scree_buf: vec![],
            pre_render_buf: vec![],
            vw: 0,
            vh: 0,
            pw: 0,
            ph: 0,
            x: 0,
            y: 0,
            task_graph: Arc::new(Mutex::new(TaskDag {})),
            new_layout: Arc::new(Mutex::new(true)),
            finished: Arc::new(Mutex::new(false)),
            layout: Layout {},
        };
        Ok(r)
    }
}

struct Layout {}

#[derive(Debug)]
pub enum RenderErrorCategory {
    EMutex,
    ETerminal,
}

#[derive(Debug)]
pub struct RenderError {
    pub category: RenderErrorCategory,
    pub detail: String,
}

trait RenderLock {
    type Inner;
    fn render_lock(&mut self) -> Result<MutexGuard<Self::Inner>, RenderError>;
}

impl<T> RenderLock for Arc<Mutex<T>> {
    type Inner = T;

    fn render_lock(&mut self) -> Result<MutexGuard<Self::Inner>, RenderError> {
        self.lock()
            .map_err(|e| RenderError {
                category: EMutex,
                detail: e.to_string(),
            })
    }
}

trait MapTermError {
    type R;
    fn map_term_err(self) -> Result<Self::R, RenderError>;
}

impl<T> MapTermError for io::Result<T> {
    type R = T;

    fn map_term_err(self) -> Result<Self::R, RenderError> {
        self.map_err(|e| RenderError {
            category: ETerminal,
            detail: e.to_string(),
        })
    }
}


impl Renderer {
    fn render_single_frame(&mut self) -> Result<(), RenderError> {
        {
            let mut new_layout = self.new_layout.render_lock()?;
            if *new_layout {
                *new_layout = false;
                drop(new_layout);
                self.detect_layout()?;
                let buf_len = self.vw * self.vh;
                if buf_len != self.pre_render_buf.len() {
                    self.pre_render_buf.resize(buf_len, '\0');
                }
                if buf_len != self.on_scree_buf.len() {
                    self.on_scree_buf.resize(buf_len, ' ');
                }

                // clear screen
                let mut stdout = stdout();
                stdout.queue(MoveTo(self.y as u16, self.x as u16)).map_term_err()?;
                for i in 0..self.vh {
                    for j in 0..self.vw {
                        stdout.queue(Print(' ')).map_term_err()?;
                    }
                    stdout.queue(MoveTo(0u16, (self.x + i) as u16)).map_term_err()?;
                }
                stdout.queue(MoveTo(self.y as u16, self.x as u16)).map_term_err()?;
                stdout.flush().map_term_err()?;
            }
        }
        self.fill_pre_render()?;
        self.commit_to_screen()?;
        Ok(())
    }

    pub async fn update_tasks(&mut self, tasks: TaskDag) -> Result<(), RenderError> {
        let mut new_layout = self.new_layout.render_lock()?;
        let mut cur_tasks = self.task_graph.render_lock()?;
        *cur_tasks = tasks;
        *new_layout = true;
        Ok(())
    }

    fn fill_pre_render(&mut self) -> Result<(), RenderError> {
        // clear first
        self.pre_render_buf.fill('\0');

        for i in 0..self.vh {
            for j in 0..self.vw {
                let flag = (i + j) % 5 == 0;
                if !flag {
                    continue;
                }
                let c = '💩';
                #[cfg(debug_assertions)]
                assert!(i * self.vw + j < self.pre_render_buf.len());
                let x = unsafe { self.pre_render_buf.get_unchecked_mut(i * self.vw + j) };
                *x = c;
            }
        }
        Ok(())
    }

    fn commit_to_screen(&mut self) -> Result<(), RenderError> {
        let mut stdout = stdout();
        stdout.queue(MoveTo(self.y as u16, self.x as u16)).map_term_err()?;
        for i in 0..self.vh {
            for j in 0..self.vw {
                #[cfg(debug_assertions)]
                assert!(i * self.vw + j < self.pre_render_buf.len());
                let c_new = *unsafe { self.pre_render_buf.get_unchecked(i * self.vw + j) };
                #[cfg(debug_assertions)]
                assert!(i * self.vw + j < self.on_scree_buf.len());
                let c_old = *unsafe { self.on_scree_buf.get_unchecked(i * self.vw + j) };
                // skip empty char because some char has 2-column width
                if c_new == '\0' {
                    continue;
                }
                // skip the same char
                if c_old == c_new {
                    continue;
                }
                let (x, y) = (self.x + i, self.y + j);
                stdout.queue(MoveTo(y as u16, x as u16)).map_term_err()?;
                stdout.queue(Print(c_new)).map_term_err()?;
            }
            stdout.queue(MoveTo(0u16, (self.x + i) as u16)).map_term_err()?;
        }
        stdout.queue(MoveTo(0u16, (self.x + self.vh) as u16)).map_term_err()?;
        stdout.flush().map_term_err()?;
        std::mem::swap(&mut self.on_scree_buf, &mut self.pre_render_buf);
        Ok(())
    }

    fn detect_layout(&mut self) -> Result<(), RenderError> {
        Ok(())
    }

    pub fn render_loop(&mut self) -> Result<(), RenderError> {
        self.detect_resize()?;
        self.update_pos()?;
        self.detect_layout()?;
        let mut cnt = 0u64;

        loop {
            self.render_single_frame()?;
            cnt += 1;
            if cnt == 30 {
                cnt = 0;

                self.detect_resize()?;

                let finished = self.finished.render_lock()?;
                if *finished {
                    break;
                }
            }
        }
        Ok(())
    }


    fn detect_resize(&mut self) -> Result<(), RenderError> {
        let (w, h) = dimensions().ok_or(RenderError {
            category: ETerminal,
            detail: "Cannot get terminal size".into(),
        })?;
        self.pw = w;
        self.ph = h;
        let (old_w, old_h) = (self.vw, self.vh);
        let border = 5usize;
        self.vw = max(w, border) - border;
        self.vh = max(h, border) - border;
        // self.vw = min(self.vw, 120);
        // self.vh = min(self.vh, 40);
        if (old_w, old_h) != (self.vw, self.vh) {
            let mut new_layout = self.new_layout.render_lock()?;
            *new_layout = true;
        }
        Ok(())
    }

    pub fn p_info(&self) {
        println!("Physical screen size: ({}, {})", self.ph, self.pw);
        println!("Virtual screen size: ({}, {})", self.vh, self.vw);
        println!("Current cursor: ({}, {})", self.x, self.y);
    }

    fn update_pos(&mut self) -> Result<(), RenderError> {
        let mut stdout = stdout();
        stdout.flush().map_term_err()?;
        #[cfg(debug_assertions)]
        assert!(self.vw != 0 && self.vh != 0);
        let (y, x) = cursor::position().map_term_err()?;
        self.x = x as usize;
        self.y = y as usize;
        if self.x + self.vh >= self.ph {
            let offset = self.x + self.vh - self.ph;
            for _ in 0..=offset + 1 {
                stdout.queue(Print('\n')).map_term_err()?;
            }
            stdout.flush().map_term_err()?;
            stdout.queue(ScrollUp(self.vh as u16)).map_term_err()?;
            self.x = self.ph - self.vh - 1;
            stdout.queue(MoveTo(0u16, self.x as u16)).map_term_err()?;
            stdout.flush().map_term_err()?;
        }
        Ok(())
    }
}