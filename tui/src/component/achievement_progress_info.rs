use std::cmp::min;

use tui::{
    style::{Color, Style},
    widgets::Gauge,
};

use orrient::api::{AccountAchievement, Achievement};

pub struct AchievementProgressInfo;

impl AchievementProgressInfo {
    pub fn new_widget<'a>(
        achievement: &'a Achievement,
        account_achievement: Option<&'a AccountAchievement>,
    ) -> Vec<Gauge<'a>> {
        if let Some(account_achievement) = account_achievement {
            achievement
                .tiers
                .iter()
                .enumerate()
                .map(|(i, tier)| {
                    let current_progress: f64 =
                        min(account_achievement.current.unwrap_or_default(), tier.count) as f64;
                    let total: f64 = tier.count as f64;
                    let ratio = current_progress / total;
                    Gauge::default()
                        .label(format!(
                            "Tier {} ({} AP): {}%",
                            i + 1,
                            tier.points,
                            ((ratio * 100f64) as u16)
                        ))
                        .ratio(ratio)
                        .gauge_style(Style::default().fg(Color::Gray).bg(Color::Black))
                })
                .collect::<Vec<Gauge>>()
        } else {
            vec![]
        }
    }
}
