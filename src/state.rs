use std::{collections::HashMap, sync::RwLock};

use crate::{
    api::{AccountAchievement, Achievement, AllAccountAchievements, Dailies},
    config::Config,
    tracks::Track,
};

pub struct AppState {
    current_tab: RwLock<usize>,
    achievements: RwLock<HashMap<usize, Achievement>>,
    account_achievements: RwLock<HashMap<usize, AccountAchievement>>,
    tracked_items: RwLock<Vec<Track>>,
    dailies: RwLock<Option<Dailies>>,
}

impl AppState {
    pub fn new(config: &Config) -> Self {
        Self {
            current_tab: RwLock::new(config.starting_tab - 1),
            achievements: RwLock::new(HashMap::default()),
            account_achievements: RwLock::new(HashMap::default()),
            tracked_items: RwLock::new(Vec::default()),
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
        if let Ok(mut tracked_items) = self.tracked_items.write() {
            if tracked_items.iter().find(|t| (*t).eq(&track)).is_none() {
                tracked_items.push(track);
            }
        }
    }

    pub fn toggle_track(&self, track: Track) {
        if let Ok(mut tracked_achievements) = self.tracked_items.write() {
            if let Some(index) = tracked_achievements.iter().position(|t| t.eq(&track)) {
                tracked_achievements.remove(index);
            } else {
                tracked_achievements.push(track);
            }
        }
    }

    pub fn tracked_items(&self) -> Vec<Track> {
        if let Ok(tracked_items) = self.tracked_items.read() {
            tracked_items.clone()
        } else {
            Vec::default()
        }
    }

    pub fn is_tracked(&self, track: Track) -> bool {
        if let Ok(tracked_items) = self.tracked_items.read() {
            tracked_items.contains(&track)
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
