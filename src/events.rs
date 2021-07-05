use std::collections::HashSet;

use crate::{
    api::{Achievement, AllAccountAchievements, Dailies},
    tracks::Track,
};

#[derive(Debug)]
pub enum Event {
    Quit,
    AddTrack(Track),
    ToggleTrack(Track),
    AccountAchievementsLoaded(AllAccountAchievements),
    AchievementsLoaded(HashSet<Achievement>),
    FetchedDailies(Dailies),
    StatusMessage(String),
}
