use std::time::Duration;

use futures::Future;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    sync::broadcast::{self, error::RecvError, Sender},
    task::JoinHandle,
    time::sleep,
};

use crate::event::Event;
use crate::store::Store;

pub trait Actor: Serialize + DeserializeOwned + Default + Send + Sync + 'static {
    const NAME: &'static str;

    fn handle(&mut self, event: Event) -> anyhow::Result<Vec<Event>>;
}

pub struct System<S> {
    store: S,
    sender: Sender<Event>,
    handles: Vec<JoinHandle<()>>,
}

impl<S> System<S> {
    pub fn new(store: S) -> Self {
        let (sender, _) = broadcast::channel(128);

        Self {
            store,
            sender,
            handles: Default::default(),
        }
    }

    pub fn abort_everything(&mut self) {
        for handle in self.handles.drain(..) {
            handle.abort()
        }
    }
}

impl<S: Store + 'static> System<S> {
    pub fn spawn<ACTOR: Actor>(&mut self) {
        let mut actor: ACTOR = self
            .store
            .read()
            .expect("Failed to read actor")
            .unwrap_or_default();

        let sender = self.sender.clone();
        let mut receiver = sender.subscribe();

        let thread_store = self.store.clone();

        let future = async move {
            loop {
                let new_events = match receiver.recv().await {
                    Ok(event) => {
                        let new_events = actor.handle(event).unwrap();

                        thread_store.write(&actor).unwrap();

                        new_events
                    }
                    Err(RecvError::Closed) => break,
                    _ => vec![],
                };

                for event in new_events {
                    sender.send(event).unwrap();
                }
            }
        };

        let handle = tokio::spawn(future);

        self.handles.push(handle);
    }

    pub fn register_io_task<F, Fut>(&mut self, task: F)
    where
        F: FnOnce(Sender<Event>) -> Fut,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let handle = tokio::spawn(task(self.sender.clone()));
        self.handles.push(handle);
    }

    pub async fn tick_and_wait(self, duration: Duration) {
        loop {
            if self.handles.iter().all(|handle| handle.is_finished()) {
                return;
            }

            self.sender.send(Event::Tick).unwrap();

            sleep(duration).await;
        }
    }

    pub fn rage_quit(mut self) -> S {
        self.abort_everything();
        self.store.clone()
    }
}

impl<S> std::ops::Drop for System<S> {
    fn drop(&mut self) {
        self.abort_everything()
    }
}

mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_system() {
        let number_of_events = 1337;
        
        let store = crate::store::MemoryStore::default();
        let mut system = System::new(store);

        system.spawn::<EventCounter>();

        for _ in 0..number_of_events {
            system.sender.send(Event::Tick).unwrap();
        }
    }

    async fn test_tick_and_wait() {
        todo!();
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct EventCounter {
        event: Event,
        count: usize,
        target_count: usize,
    }

    impl Actor for EventCounter {
        const NAME: &'static str = "EventCounter";

        fn handle(&mut self, event: Event) -> anyhow::Result<Vec<Event>> {
            self.count += 1;
            Ok(vec![])
        }
    }
}
