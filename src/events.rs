use std::collections::HashSet;

use crate::{
    api::{Achievement, AllAccountAchievements, Dailies},
    bookmarks::Bookmark,
    tracks::Track,
};

#[derive(Debug)]
pub enum Event {
    Quit,
    AddTrack(Track),
    AddBookmark(Bookmark),
    ToggleTrack(Track),
    AccountAchievementsLoaded(AllAccountAchievements),
    AchievementsLoaded(HashSet<Achievement>),
    FetchedDailies(Dailies),
    StatusMessage(String),
}
