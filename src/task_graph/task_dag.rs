use std::sync::Arc;

#[derive(Default)]
pub struct TaskDag {}

impl TaskDag {
    pub fn to_arc(self) -> Arc<Self> {
        Arc::new(self)
    }
}