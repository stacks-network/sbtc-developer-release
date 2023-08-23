use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::actor::Actor;

pub trait Store: Send + Clone {
    type Error: std::fmt::Debug;
    fn read<ACTOR: Actor>(&self) -> Result<Option<ACTOR>, Self::Error>;
    fn write<ACTOR: Actor>(&self, obj: &ACTOR) -> Result<(), Self::Error>;
}

#[derive(Debug, Default, Clone)]
pub struct MemoryStore {
    actors: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl<'a> Store for MemoryStore {
    type Error = anyhow::Error;

    fn read<ACTOR: Actor>(&self) -> Result<Option<ACTOR>, Self::Error> {
        let actors = self.actors.lock().unwrap();

        let Some(buffer) = actors.get(ACTOR::NAME) else {
            return Ok(None)
        };

        let actor = serde_json::from_reader(buffer.as_slice())?;

        Ok(Some(actor))
    }

    fn write<ACTOR: Actor>(&self, obj: &ACTOR) -> Result<(), Self::Error> {
        let mut bytes = Vec::new();
        serde_json::to_writer(&mut bytes, obj)?;

        self.actors
            .lock()
            .unwrap()
            .insert(ACTOR::NAME.to_string(), bytes);

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FileStore {
    state_directory: PathBuf,
}

impl FileStore {
    pub fn new(state_directory: PathBuf) -> Self {
        std::fs::create_dir_all(state_directory.clone()).unwrap();
        Self { state_directory }
    }

    fn save_file(&self, name: &str) -> PathBuf {
        self.state_directory.clone().join(format!("{}.json", name))
    }
}

impl Store for FileStore {
    type Error = anyhow::Error;

    fn read<ACTOR: Actor>(&self) -> Result<Option<ACTOR>, Self::Error> {
        let file_result = std::fs::File::open(&self.save_file(ACTOR::NAME));

        let file = match file_result {
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    return Ok(None);
                } else {
                    return Err(anyhow::Error::from(err));
                }
            }
            Ok(file) => file,
        };

        Ok(Some(serde_json::from_reader(file)?))
    }

    fn write<ACTOR: Actor>(&self, obj: &ACTOR) -> Result<(), Self::Error> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.save_file(ACTOR::NAME))
            .unwrap();

        serde_json::to_writer_pretty(file, obj)?;

        Ok(())
    }
}
