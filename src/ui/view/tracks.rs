use std::{collections::HashMap, rc::Rc};

use crate::{
    api::{AccountAchievement, Achievement},
    events::{CursorMovement, Event, InputEvent, InputKind, StateEvent, ViewEvent},
    state::AppState,
    tracks::Track,
    ui::{
        component::{
            achievement_info::AchievementInfo, achievement_progress_info::AchievementProgressInfo,
        },
        widget::list_selection::ListSelection,
    },
};
use tokio::sync::mpsc::UnboundedSender;
use tui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use super::View;

pub struct TracksView {
    app_state: Rc<AppState>,
    tx_state: UnboundedSender<Event>,
    list_state: ListState,
    tier_progress_bar_height: u16,
    achievements: HashMap<usize, Achievement>,
    account_achievements: HashMap<usize, AccountAchievement>,
    tracks: Vec<usize>,
}

impl TracksView {
    pub fn new(app_state: Rc<AppState>, tx_state: UnboundedSender<Event>) -> Self {
        TracksView {
            app_state,
            tx_state,
            list_state: ListState::default(),
            tier_progress_bar_height: 1,
            achievements: HashMap::default(),
            account_achievements: HashMap::default(),
            tracks: Vec::default(),
        }
    }

    fn selected_id(&self) -> Option<usize> {
        if let Some(selected_index) = self.list_state.selected() {
            self.tracks.get(selected_index).map(ToOwned::to_owned)
        } else {
            None
        }
    }

    fn draw_sidebar<B: tui::backend::Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        frame.render_stateful_widget(
            List::new(
                self.tracks
                    .iter()
                    .map(|id| {
                        let account_achievement = self.account_achievements.get(id);
                        let current = account_achievement.as_ref().map(|aa| aa.current).flatten();
                        let max = account_achievement.map(|aa| aa.max).flatten();
                        let percent_complete = if let (Some(current), Some(max)) = (current, max) {
                            Some(((current as f64) / (max as f64) * 100f64) as u16)
                        } else {
                            None
                        };
                        let achievement_name = self
                            .achievements
                            .get(id)
                            .map(|a| a.name.clone())
                            .unwrap_or_default();
                        ListItem::new(percent_complete.map_or(
                            achievement_name.to_string(),
                            |percent_complete| {
                                format!("({:>3}%) {}", percent_complete, achievement_name)
                            },
                        ))
                    })
                    .collect::<Vec<ListItem>>(),
            )
            .block(Block::default().borders(Borders::RIGHT))
            .highlight_symbol(">>"),
            area,
            &mut self.list_state,
        );
    }

    fn draw_progress<B: tui::backend::Backend>(
        &self,
        frame: &mut Frame<B>,
        area: Rect,
        achievement: &Achievement,
        account_achievement: Option<&AccountAchievement>,
    ) {
        let constraints = achievement
            .tiers
            .iter()
            .map(|_| Constraint::Min(self.tier_progress_bar_height))
            .collect::<Vec<Constraint>>();

        let gauge_chunk: Vec<Rect> = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        AchievementProgressInfo::new_widget(achievement, account_achievement)
            .into_iter()
            .enumerate()
            .for_each(|(i, gauge)| {
                frame.render_widget(gauge, gauge_chunk[i]);
            });
    }

    fn draw_info<B: tui::backend::Backend>(
        &self,
        frame: &mut Frame<B>,
        area: Rect,
        achievement: &Achievement,
        account_achievement: Option<&AccountAchievement>,
    ) {
        frame.render_widget(
            AchievementInfo::new_widget(achievement, account_achievement),
            area,
        );
    }
}

impl View for TracksView {
    fn draw<B: tui::backend::Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        let h_chunks: Vec<Rect> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(50), Constraint::Percentage(100)])
            .split(area);
        self.draw_sidebar(frame, h_chunks[0]);

        if let Some(achievement) = self
            .selected_id()
            .map(|achievement_id| self.achievements.get(&achievement_id))
            .flatten()
        {
            let account_achievement = self.account_achievements.get(&achievement.id);

            let progress_height = achievement.tiers.len() as u16 * self.tier_progress_bar_height;

            let info_chunks: Vec<Rect> = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(progress_height),
                    Constraint::Percentage(100),
                ])
                .split(h_chunks[1].inner(&Margin {
                    horizontal: 0,
                    vertical: 0,
                }));

            self.draw_progress(frame, info_chunks[0], achievement, account_achievement);
            self.draw_info(frame, info_chunks[1], achievement, account_achievement)
        }
    }

    fn handle_input_event(&mut self, event: &InputEvent) -> bool {
        match event.input {
            InputKind::MoveUp(amount) => {
                self.list_state.move_cursor(
                    self.app_state.tracked_achievements().len(),
                    CursorMovement::Up(amount),
                );
                true
            }
            InputKind::MoveDown(amount) => {
                self.list_state.move_cursor(
                    self.app_state.tracked_achievements().len(),
                    CursorMovement::Down(amount),
                );
                true
            }
            InputKind::Track => {
                if let Some(id) = self.selected_id() {
                    let _ = self.tx_state.send(Event::State(StateEvent::ToggleTrack(
                        Track::Achievement(id),
                    )));
                }
                true
            }
            _ => false,
        }
    }

    fn handle_view_event(&mut self, event: &ViewEvent) {
        match event {
            ViewEvent::UpdateAchievements => {
                self.achievements = self.app_state.achievements();
            }
            ViewEvent::UpdateAccountAchievements => {
                self.account_achievements = self.app_state.account_achievements();
            }
            ViewEvent::UpdateTracks => {
                self.tracks = self.app_state.tracked_achievements();
                self.list_state
                    .move_cursor(self.tracks.len(), CursorMovement::None);
            }
            _ => {}
        }
    }
}
