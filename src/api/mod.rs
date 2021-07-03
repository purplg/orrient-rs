//! The models of relevant API responses from the official Guild Wars 2 gateway
//!
//! [Official documentation](https://wiki.guildwars2.com/wiki/API:Main)

use std::{collections::HashSet, hash::Hash};

pub mod endpoints;

/// A list of all available IDs of achievements
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AllAchievementIDs(pub Vec<usize>);

/// Data about a specific achievement
#[derive(Serialize, Deserialize, Eq, Clone, Debug)]
pub struct Achievement {
    pub id: usize,
    pub icon: Option<String>,
    pub name: String,
    pub description: String,
    pub requirement: String,
    pub locked_text: String,
    #[serde(alias = "type")]
    pub achievement_type: String,
    pub flags: Vec<String>,
    pub tiers: Vec<AchievementTier>,
    pub prerequisites: Option<Vec<usize>>,
    pub rewards: Option<Vec<Reward>>,
    pub bits: Option<Vec<AchievementBit>>,
    pub point_cap: Option<i32>,
}

impl PartialEq for Achievement {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Achievement {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

/// A list of all achievements with progress on the users account
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AllAccountAchievements(pub HashSet<AccountAchievement>);

/// Progress by the user for a specific achievement
#[derive(Serialize, Deserialize, Eq, Clone, Debug)]
pub struct AccountAchievement {
    pub id: usize,
    pub bits: Option<Vec<usize>>,
    pub current: Option<usize>,
    pub max: Option<usize>,
    pub done: bool,
    pub repeated: Option<usize>,
    pub unlocked: Option<bool>,
}

impl PartialEq for AccountAchievement {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for AccountAchievement {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct AchievementTier {
    pub count: usize,
    pub points: usize,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(tag = "type")]
pub enum Reward {
    Coins { count: usize },
    Item { id: usize, count: usize },
    Mastery { id: usize, region: String },
    Title { id: usize },
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct AchievementBit {
    #[serde(alias = "type")]
    pub bit_type: Option<String>,
    pub id: Option<usize>,
    pub text: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Dailies {
    pub pve: Vec<Daily>,
    pub pvp: Vec<Daily>,
    pub wvw: Vec<Daily>,
    pub fractals: Vec<Daily>,
    pub special: Vec<Daily>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Daily {
    pub id: usize,
    pub level: LevelRange,
    pub required_access: Option<RequiredAccess>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LevelRange {
    min: usize,
    max: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RequiredAccess {
    product: Product,
    condition: AccessCondition,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Product {
    HeartOfThorns,
    PathOfFire,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AccessCondition {
    HasAccess,
    NoAccess,
}
