use std::{
    collections::{HashMap, HashSet},
    sync::RwLock,
};

use crate::{
    api::{AccountAchievement, Achievement, AllAccountAchievements, Dailies},
    tracks::{Track, Tracks},
};

pub struct AppState {
    achievements: RwLock<HashMap<usize, Achievement>>,
    account_achievements: RwLock<HashMap<usize, AccountAchievement>>,
    dailies: RwLock<Option<Dailies>>,
    tracks: RwLock<Tracks>,
}

impl AppState {
    pub fn new(tracks: Tracks) -> Self {
        Self {
            achievements: RwLock::new(HashMap::default()),
            account_achievements: RwLock::new(HashMap::default()),
            dailies: RwLock::new(None),
            tracks: RwLock::new(tracks),
        }
    }

    pub fn achievements(&self) -> HashMap<usize, Achievement> {
        if let Ok(achievements) = self.achievements.read() {
            achievements.clone()
        } else {
            HashMap::default()
        }
    }

    pub fn insert_achievements(&self, new_achievements: Vec<Achievement>) {
        if let Ok(mut achievements) = self.achievements.write() {
            for achievement in new_achievements {
                achievements.insert(achievement.id, achievement);
            }
        }
    }

    pub fn account_achievements(&self) -> HashMap<usize, AccountAchievement> {
        if let Ok(account_achievements) = self.account_achievements.read() {
            account_achievements.clone()
        } else {
            HashMap::default()
        }
    }

    pub fn set_account_achievements(&self, all_account_achievements: AllAccountAchievements) {
        if let Ok(mut account_achievements) = self.account_achievements.write() {
            *account_achievements = all_account_achievements
                .0
                .iter()
                .map(|account_achievement| (account_achievement.id, account_achievement.clone()))
                .collect::<HashMap<usize, AccountAchievement>>();
        }
    }

    pub fn add_track(&self, track: Track) {
        if let Ok(mut tracks) = self.tracks.write() {
            tracks.insert(track);
        }
        if let Ok(tracks) = self.tracks.read() {
            let _ = tracks.write();
        }
    }

    pub fn toggle_track(&self, track: Track) {
        if let Ok(mut tracks) = self.tracks.write() {
            if !tracks.remove(&track) {
                tracks.insert(track);
            }
        }
        if let Ok(tracks) = self.tracks.read() {
            let _ = tracks.write();
        }
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

    pub fn set_dailies(&self, new_dailies: Dailies) {
        if let Ok(mut dailies) = self.dailies.write() {
            *dailies = Some(new_dailies);
        }
    }

    pub fn dailies(&self) -> Option<Dailies> {
        if let Ok(dailies) = self.dailies.read() {
            dailies.clone()
        } else {
            None
        }
    }
}
