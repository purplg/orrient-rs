use std::cmp::min;

use tui::{
    layout::Constraint,
    style::{Modifier, Style},
    widgets::{Row, Table},
};

use orrient::api::{AccountAchievement, Achievement, Reward};

pub struct AchievementInfo;

impl AchievementInfo {
    pub fn new_widget<'a>(
        achievement: &'a Achievement,
        account_achievement: Option<&'a AccountAchievement>,
    ) -> Table<'a> {
        let mut rows = vec![];
        rows.append(&mut vec![
            Self::id_row(achievement),
            Self::icon_row(achievement),
            Self::description_row(achievement),
            Self::requirement_row(achievement),
            Self::locked_text_row(achievement),
            Self::type_row(achievement),
            Self::flags_row(achievement),
            Self::tiers_row(achievement),
            Self::prerequirsites_row(achievement),
            Self::rewards_row(achievement),
            Self::bits_row(achievement),
            Self::point_cap_row(achievement),
        ]);
        if let Some(account_achievement) = account_achievement {
            rows.push(Self::empty_row());
            rows.push(Self::custom_row("Progress"));
            rows.append(&mut Self::progress_rows(achievement, account_achievement));
        }

        Table::new(rows).widths(&[Constraint::Min(15), Constraint::Min(100)])
    }

    fn empty_row() -> Row<'static> {
        Row::new(vec![""])
    }

    fn custom_row<'a>(msg: &'a str) -> Row<'static> {
        Row::new(vec![String::from(msg)]).style(Style::default().add_modifier(Modifier::BOLD))
    }

    fn id_row(achievement: &Achievement) -> Row {
        Row::new(vec!["ID".to_string(), achievement.id.to_string()])
    }

    fn icon_row(achievement: &Achievement) -> Row {
        Row::new(vec![
            "Icon".to_string(),
            achievement
                .icon
                .as_ref()
                .map_or("None".to_string(), |icon| icon.to_string()),
        ])
    }

    fn description_row(achievement: &Achievement) -> Row {
        Row::new(vec![
            "Description".to_string(),
            achievement.description.to_string(),
        ])
    }

    fn requirement_row(achievement: &Achievement) -> Row {
        Row::new(vec![
            "Requirement".to_string(),
            achievement.requirement.to_string(),
        ])
    }

    fn locked_text_row(achievement: &Achievement) -> Row {
        Row::new(vec![
            "Locked Text".to_string(),
            achievement.locked_text.to_string(),
        ])
    }

    fn type_row(achievement: &Achievement) -> Row {
        Row::new(vec![
            "Type".to_string(),
            achievement.achievement_type.to_string(),
        ])
    }

    fn flags_row(achievement: &Achievement) -> Row {
        Row::new(vec!["Flags".to_string(), achievement.flags.join(", ")])
    }

    fn tiers_row(achievement: &Achievement) -> Row {
        Row::new(vec![
            "Tiers".to_string(),
            achievement
                .tiers
                .iter()
                .map(|tier| format!("{}: {}AP", tier.count, tier.points))
                .collect::<Vec<String>>()
                .join(", "),
        ])
    }

    fn prerequirsites_row(achievement: &Achievement) -> Row {
        Row::new(vec![
            "Prerequisites".to_string(),
            achievement
                .prerequisites
                .as_ref()
                .map_or("None".to_string(), |prereqs| {
                    prereqs
                        .iter()
                        .map(|id| id.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                }),
        ])
    }

    fn rewards_row(achievement: &Achievement) -> Row {
        Row::new(vec![
            "Rewards".to_string(),
            achievement
                .rewards
                .as_ref()
                .map_or("None".to_string(), |rewards| {
                    rewards
                        .iter()
                        .map(|reward| match reward {
                            Reward::Coins { count } => format!("{} coins", count),
                            Reward::Item { id, count } => format!("{}x Item {}", count, id),
                            Reward::Mastery { id, region } => {
                                format!("{} Mastery id: {}", region, id)
                            }
                            Reward::Title { id } => format!("Title id {}", id),
                        })
                        .collect::<Vec<String>>()
                        .join(", ")
                }),
        ])
    }

    fn bits_row(achievement: &Achievement) -> Row {
        Row::new(vec![
            "Bits".to_string(),
            achievement
                .bits
                .as_ref()
                .map_or("None".to_string(), |bits| {
                    bits.iter()
                        .filter_map(|bit| bit.bit_type.clone())
                        .collect::<Vec<String>>()
                        .join(", ")
                }),
        ])
    }

    fn point_cap_row(achievement: &Achievement) -> Row {
        Row::new(vec![
            "Point Cap".to_string(),
            achievement
                .point_cap
                .as_ref()
                .map_or("None".to_string(), |point_cap| point_cap.to_string()),
        ])
    }

    fn progress_rows<'a>(
        achievement: &Achievement,
        account_achievement: &AccountAchievement,
    ) -> Vec<Row<'a>> {
        achievement
            .tiers
            .iter()
            .enumerate()
            .map(|(tier_index, tier)| {
                Row::new(vec![
                    format!("Tier {} ({} AP)", tier_index + 1, tier.points),
                    format!(
                        "{}/{}",
                        min(account_achievement.current.unwrap(), tier.count),
                        tier.count
                    ),
                ])
            })
            .collect::<Vec<Row>>()
    }
}
