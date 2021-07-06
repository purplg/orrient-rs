use std::{
    collections::{hash_set::IntoIter, HashSet},
    hash::Hash,
    ops::{Deref, DerefMut},
};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Bookmarks(HashSet<Bookmark>);

impl IntoIterator for Bookmarks {
    type Item = Bookmark;
    type IntoIter = IntoIter<Bookmark>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Deref for Bookmarks {
    type Target = HashSet<Bookmark>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Bookmarks {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Serialize, Deserialize, Eq, Clone, Debug)]
pub struct Bookmark {
    pub kind: BookmarkKind,
    pub name: String,
    pub link: String,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
pub enum BookmarkKind {
    Waypoint,
}

impl PartialEq for Bookmark {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl Hash for Bookmark {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

impl Bookmarks {
    pub fn items(&self) -> &HashSet<Bookmark> {
        &self.0
    }
}
