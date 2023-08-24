use crate::event::Event;
use crate::task::Task;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct State {}

pub fn update(state: State, event: Event) -> (State, Vec<Task>) {
    todo!();
}
