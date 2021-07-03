use std::{
    collections::{hash_set::IntoIter, HashSet},
    ops::{Deref, DerefMut},
};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Tracks(HashSet<Track>);

impl IntoIterator for Tracks {
    type Item = Track;
    type IntoIter = IntoIter<Track>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Deref for Tracks {
    type Target = HashSet<Track>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Tracks {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
    pub fn items(&self) -> &HashSet<Track> {
        &self.0
    }
}
