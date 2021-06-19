use super::{AccountAchievement, Achievement, AllAccountAchievements, AllAchievementIDs, Dailies};

/// Represents how and where to access the requested data
pub trait Endpoint<P> {
    /// Whether the endpoint requires an API key from the user
    const AUTHENTICATED: bool;

    /// Build a url path to the endpoint from the provided parameters
    fn get_path(param: Vec<&P>) -> String;
}

impl Endpoint<()> for AllAchievementIDs {
    const AUTHENTICATED: bool = false;

    fn get_path(_: Vec<&()>) -> String {
        "v2/achievements".to_string()
    }
}

impl Endpoint<usize> for Achievement {
    const AUTHENTICATED: bool = false;

    fn get_path(ids: Vec<&usize>) -> String {
        format!(
            "v2/achievements?ids={}",
            ids.iter()
                .map(|id| id.to_string())
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

impl Endpoint<usize> for AccountAchievement {
    const AUTHENTICATED: bool = true;

    fn get_path(ids: Vec<&usize>) -> String {
        format!(
            "v2/account/achievements?ids={}",
            ids.iter()
                .map(|id| id.to_string())
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

impl Endpoint<()> for AllAccountAchievements {
    const AUTHENTICATED: bool = true;

    fn get_path(_: Vec<&()>) -> String {
        "v2/account/achievements".to_string()
    }
}

impl Endpoint<()> for Dailies {
    const AUTHENTICATED: bool = false;

    fn get_path(_: Vec<&()>) -> String {
        "v2/achievements/daily".to_string()
    }
}
