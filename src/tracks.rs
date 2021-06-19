use std::{fs::File, io::BufWriter, marker::PhantomData, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Track {
    Achievement(usize),
}

impl PartialEq for Track {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Track {
    pub fn id(&self) -> usize {
        match self {
            Track::Achievement(id) => *id,
        }
    }
}

impl Reader<'_, Track> for Track {
    fn load(path: &str) -> (Vec<Track>, Writer<Vec<Track>>) {
        let path = PathBuf::from(path);
        (
            match File::open(&path) {
                Ok(file) => match serde_json::from_reader(&file) {
                    Ok(cache) => cache,
                    Err(_) => Vec::default(),
                },
                Err(_) => Vec::default(),
            },
            Writer {
                path,
                _phantom_data: PhantomData::default(),
            },
        )
    }
}

pub trait Reader<'de, T>
where
    T: Deserialize<'de> + Serialize,
{
    fn load(path: &str) -> (Vec<T>, Writer<Vec<T>>);
}

pub struct Writer<T: Serialize> {
    path: PathBuf,
    _phantom_data: PhantomData<T>,
}

impl<T: Serialize> Writer<T> {
    pub fn write(&self, content: T) -> Result<(), Box<dyn std::error::Error>> {
        let bw = BufWriter::new(File::create(&self.path)?);
        let _ = serde_json::to_writer(bw, &content)?;
        Ok(())
    }
}
