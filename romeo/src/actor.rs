use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
    time::Duration,
};

use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    sync::broadcast::{self, error::RecvError, Sender},
    task::JoinHandle,
    time::sleep,
};

use crate::event::Event;

pub trait Actor: Serialize + DeserializeOwned + Default + Send + Sync + 'static {
    const NAME: &'static str;

    fn handle(&mut self, event: Event) -> anyhow::Result<Vec<Event>>;

    fn save(&self, writer: impl std::io::Write) -> anyhow::Result<()> {
        serde_json::to_writer_pretty(writer, self)?;

        Ok(())
    }

    fn load(reader: impl std::io::Read) -> anyhow::Result<Self> {
        let self_ = serde_json::from_reader(reader)?;

        Ok(self_)
    }
}

pub struct System {
    state_directory: PathBuf,
    sender: Sender<Event>,
    handles: Vec<JoinHandle<()>>,
}

impl System {
    pub fn new(state_directory: impl AsRef<Path>) -> Self {
        let (sender, _) = broadcast::channel(128);

        Self {
            state_directory: state_directory.as_ref().to_path_buf(),
            sender,
            handles: Default::default(),
        }
    }

    pub fn spawn<ACTOR: Actor>(&mut self) {
        let save_file = self
            .state_directory
            .clone()
            .join(format!("{}.json", ACTOR::NAME));

        let save_file_read = std::fs::File::open(&save_file).unwrap();
        let mut actor = ACTOR::load(&save_file_read).unwrap_or_default();

        let sender = self.sender.clone();
        let mut receiver = sender.subscribe();

        self.handles.push(tokio::spawn(async move {
            loop {
                let new_events = match receiver.recv().await {
                    Ok(event) => {
                        let new_events = actor.handle(event).unwrap();

                        let save_file_handle = std::fs::OpenOptions::new()
                            .create(true)
                            .write(true)
                            .truncate(true)
                            .open(&save_file)
                            .unwrap();

                        actor.save(&save_file_handle).unwrap();

                        new_events
                    }
                    Err(RecvError::Closed) => break,
                    _ => vec![],
                };

                for event in new_events {
                    sender.send(event).unwrap();
                }
            }
        }));
    }

    pub async fn tick_and_wait(self, duration: Duration) {
        create_dir_all(self.state_directory).unwrap();

        loop {
            if self.handles.iter().all(|handle| handle.is_finished()) {
                return;
            }

            self.sender.send(Event::Tick).unwrap();

            sleep(duration).await;
        }
    }
}

mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_stuff() {
        todo!();
    }
}
