use std::{collections::HashSet, fs::File, io::BufWriter, path::PathBuf, sync::RwLock};

use crate::tracks::{Track, Tracks};

#[derive(Serialize, Deserialize, Default)]
pub struct AppState {
    #[serde(skip)]
    path: PathBuf,
    tracks: RwLock<Tracks>,
}

impl AppState {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            tracks: RwLock::new(Tracks::default()),
        }
    }

    pub fn load(path: &str) -> Self {
        let path = PathBuf::from(path);
        match File::open(&path) {
            Ok(file) => match serde_json::from_reader(&file) {
                Ok(state) => state,
                Err(_) => Self::new(path),
            },
            Err(_) => Self::new(path),
        }
    }

    pub fn add_track(&self, track: Track) {
        if let Ok(mut tracks) = self.tracks.write() {
            tracks.insert(track);
        }
        let _ = self.write();
    }

    pub fn toggle_track(&self, track: Track) {
        if let Ok(mut tracks) = self.tracks.write() {
            if !tracks.remove(&track) {
                tracks.insert(track);
            }
        }
        let _ = self.write();
    }

    pub fn tracked_items(&self) -> HashSet<Track> {
        if let Ok(tracks) = self.tracks.read() {
            tracks.items().clone()
        } else {
            HashSet::default()
        }
    }

    pub fn is_tracked(&self, track: &Track) -> bool {
        if let Ok(tracks) = self.tracks.read() {
            tracks.contains(track)
        } else {
            false
        }
    }

    pub fn write(&self) -> Result<(), Box<dyn std::error::Error>> {
        let bw = BufWriter::new(File::create(&self.path)?);
        let _ = serde_json::to_writer(bw, &self)?;
        Ok(())
    }
}
