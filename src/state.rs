use std::{collections::HashMap, sync::RwLock};

use crate::{
    api::{AccountAchievement, Achievement, AllAccountAchievements, Dailies},
    tracks::Track,
};

pub struct AppState {
    current_tab: RwLock<usize>,
    achievements: RwLock<HashMap<usize, Achievement>>,
    account_achievements: RwLock<HashMap<usize, AccountAchievement>>,
    tracked_achievements: RwLock<Vec<usize>>,
    status_message: RwLock<String>,
    dailies: RwLock<Option<Dailies>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_tab: RwLock::new(0),
            achievements: RwLock::new(HashMap::default()),
            account_achievements: RwLock::new(HashMap::default()),
            tracked_achievements: RwLock::new(Vec::default()),
            status_message: RwLock::new(String::default()),
            dailies: RwLock::new(None),
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
        if let Ok(mut tracked_achievements) = self.tracked_achievements.write() {
            if tracked_achievements
                .iter()
                .position(|t| *t == track.id())
                .is_none()
            {
                tracked_achievements.push(track.id());
            }
        }
    }

    pub fn toggle_track(&self, track: Track) {
        if let Ok(mut tracked_achievements) = self.tracked_achievements.write() {
            if let Some(index) = tracked_achievements.iter().position(|t| *t == track.id()) {
                tracked_achievements.remove(index);
            } else {
                tracked_achievements.push(track.id());
            }
        }
    }

    pub fn tracked_achievements(&self) -> Vec<usize> {
        if let Ok(tracked_achievements) = self.tracked_achievements.read() {
            tracked_achievements.clone()
        } else {
            Vec::default()
        }
    }

    pub fn is_tracked(&self, id: Option<usize>) -> bool {
        if let (Some(id), Ok(tracked_achievements)) = (id, self.tracked_achievements.read()) {
            tracked_achievements.contains(&id)
        } else {
            false
        }
    }

    pub fn select_tab(&self, tab_index: usize) {
        if let Ok(mut current_tab) = self.current_tab.write() {
            *current_tab = tab_index;
        }
    }

    pub fn current_tab(&self) -> Option<usize> {
        match self.current_tab.read() {
            Ok(current_tab) => Some(*current_tab),
            _ => None,
        }
    }

    pub fn set_status(&self, message: String) {
        if let Ok(mut status_message) = self.status_message.write() {
            *status_message = message;
        }
    }

    pub fn status(&self) -> String {
        if let Ok(status_message) = self.status_message.read() {
            status_message.clone()
        } else {
            String::default()
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
