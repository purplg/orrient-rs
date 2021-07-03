use std::{
    collections::{hash_set::IntoIter, HashSet},
    fs::File,
    io::BufWriter,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Tracks {
    path: PathBuf,
    tracks: HashSet<Track>,
}

impl IntoIterator for Tracks {
    type Item = Track;
    type IntoIter = IntoIter<Track>;

    fn into_iter(self) -> Self::IntoIter {
        self.tracks.into_iter()
    }
}

impl Deref for Tracks {
    type Target = HashSet<Track>;

    fn deref(&self) -> &Self::Target {
        &self.tracks
    }
}

impl DerefMut for Tracks {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tracks
    }
}

#[derive(Serialize, Deserialize, Hash, Eq, Clone, Debug)]
pub enum Track {
    Achievement(usize),
    Custom(String),
}

impl PartialEq for Track {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Track::Achievement(id) => {
                if let Track::Achievement(other_id) = other {
                    return id == other_id;
                }
                return false;
            }
            Track::Custom(content) => {
                if let Track::Custom(other_content) = other {
                    return content == other_content;
                }
                return false;
            }
        }
    }
}

impl Tracks {
    pub fn load(path: &str) -> Self {
        let path = PathBuf::from(path);
        let tracks = match File::open(&path) {
            Ok(file) => match serde_json::from_reader(&file) {
                Ok(cache) => cache,
                Err(_) => HashSet::default(),
            },
            Err(_) => HashSet::default(),
        };
        Self { path, tracks }
    }

    pub fn write(&self) -> Result<(), Box<dyn std::error::Error>> {
        let bw = BufWriter::new(File::create(&self.path)?);
        let _ = serde_json::to_writer(bw, &self.tracks)?;
        Ok(())
    }

    pub fn items(&self) -> &HashSet<Track> {
        &self.tracks
    }
}
