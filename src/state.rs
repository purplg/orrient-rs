use std::{
    cell::Cell,
    collections::HashSet,
    fs::File,
    io::{BufWriter, Read, Write},
    path::PathBuf,
    sync::RwLock,
};

use log::debug;

use crate::{
    bookmarks::{Bookmark, Bookmarks},
    tracks::{Track, Tracks},
};

#[derive(Serialize, Deserialize, Default)]
pub struct AppState {
    #[serde(skip)]
    path: PathBuf,
    #[serde(skip)]
    invalidated: Cell<bool>,
    tracks: RwLock<Tracks>,
    bookmarks: RwLock<Bookmarks>,
}

impl AppState {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            invalidated: Cell::new(false),
            tracks: RwLock::new(Tracks::default()),
            bookmarks: RwLock::new(Bookmarks::default()),
        }
    }

    pub fn load(path: &str) -> Self {
        let path = PathBuf::from(path);
        let mut content = String::default();

        if let Err(err) = File::open(&path).and_then(|mut file| file.read_to_string(&mut content)) {
            debug!("Error reading state file: {}", err);
            return Self::new(path);
        }

        match ron::from_str::<Self>(&content) {
            Ok(state) => state,
            Err(err) => {
                debug!("Error parsing state file: {}", err);
                Self::new(path)
            }
        }
    }

    pub fn add_bookmark(&self, bookmark: Bookmark) {
        if let Ok(mut bookmarks) = self.bookmarks.write() {
            if bookmarks.insert(bookmark) {
                self.invalidated.set(true);
            }
        }
        self.write_invalid();
    }

    pub fn remove_bookmark(&self, bookmark: Bookmark) {
        if let Ok(mut bookmarks) = self.bookmarks.write() {
            if bookmarks.remove(&bookmark) {
                self.invalidated.set(true);
            }
        }
        self.write_invalid();
    }

    pub fn bookmarks(&self) -> HashSet<Bookmark> {
        if let Ok(bookmarks) = self.bookmarks.read() {
            bookmarks.items().clone()
        } else {
            HashSet::default()
        }
    }

    pub fn toggle_track(&self, track: &Track) {
        if let Ok(mut tracks) = self.tracks.write() {
            if !tracks.remove(track) {
                if tracks.insert(track.clone()) {
                    self.invalidated.set(true);
                }
            } else {
                self.invalidated.set(true);
            }
        }
        self.write_invalid();
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

    fn write_invalid(&self) {
        if self.invalidated.get() {
            match self.write() {
                Ok(_) => self.invalidated.set(false),
                Err(err) => debug!("Error writing state file: {}", err),
            }
        }
    }

    fn write(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut bw = BufWriter::new(File::create(&self.path)?);
        if let Err(err) = bw.write_all(ron::to_string(self)?.as_bytes()) {
            debug!("Error writing state to disk: {}", err)
        }
        Ok(())
    }
}
