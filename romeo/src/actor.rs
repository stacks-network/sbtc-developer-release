use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    sync::broadcast::{error::RecvError, Sender},
    task::JoinHandle,
};

use crate::event::Event;

pub trait Actor: Serialize + DeserializeOwned + Default + Send + Sync + 'static {
    const NAME: &'static str;

    fn handle(&mut self, event: Event) -> anyhow::Result<Vec<Event>>;

    fn save(&self, path: impl AsRef<std::path::Path>) -> anyhow::Result<()> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;

        serde_json::to_writer_pretty(file, self)?;

        Ok(())
    }

    fn load(path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        let file = std::fs::File::open(path)?;
        let self_ = serde_json::from_reader(file)?;

        Ok(self_)
    }
}

pub fn spawn<A: Actor>(sender: &Sender<Event>) -> JoinHandle<()> {
    let mut actor = A::load(".").unwrap_or_default();

    let sender = sender.clone();
    let mut receiver = sender.subscribe();

    tokio::spawn(async move {
        loop {
            let new_events = match receiver.recv().await {
                Ok(event) => {
                    let new_events = actor.handle(event).unwrap();

                    let save_file = format!("./{}.json", A::NAME);
                    actor.save(save_file).unwrap();

                    new_events
                }
                Err(RecvError::Closed) => break,
                _ => vec![],
            };

            for event in new_events {
                sender.send(event).unwrap();
            }
        }
    })
}
