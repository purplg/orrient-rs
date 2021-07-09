pub mod popup;

use crate::{
    input::{InputEvent, InputKind},
    ui::{
        component::{
            achievement_info::AchievementInfo, achievement_progress_info::AchievementProgressInfo,
        },
        view::tracks::popup::CustomTrackPopupState,
        widget::list_selection::CursorMovement,
    },
};
use std::{collections::HashMap, io::Stdout, rc::Rc};

use crate::{
    api::{AccountAchievement, Achievement},
    events::Event,
    state::AppState,
    tracks::Track,
    ui::widget::list_selection::ListSelection,
};

use tokio::sync::mpsc::UnboundedSender;

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use super::View;

pub struct TracksView {
    app_state: Rc<AppState>,
    tx_event: UnboundedSender<Event>,
    list_state: ListState,
    tier_progress_bar_height: u16,
    achievements: HashMap<usize, Achievement>,
    account_achievements: HashMap<usize, AccountAchievement>,
    tracks: Vec<Track>,
    add_track_popup: CustomTrackPopupState,
}

impl TracksView {
    pub fn new(app_state: Rc<AppState>, tx_event: UnboundedSender<Event>) -> Self {
        let tracks: Vec<Track> = app_state.tracked_items().into_iter().collect();
        let mut list_state = ListState::default();
        list_state.move_cursor(tracks.len(), CursorMovement::None);
        let add_track_popup = CustomTrackPopupState::new(tx_event.clone());
        TracksView {
            app_state,
            tx_event,
            list_state,
            tier_progress_bar_height: 1,
            achievements: HashMap::default(),
            account_achievements: HashMap::default(),
            tracks,
            add_track_popup,
        }
    }

    fn selected_track(&self) -> Option<Track> {
        if let Some(selected_index) = self.list_state.selected() {
            self.tracks.get(selected_index).map(ToOwned::to_owned)
        } else {
            None
        }
    }

    fn new_list_item<'a>(&self, track: &'a Track) -> ListItem<'a> {
        match track {
            Track::Achievement(id) => {
                let account_achievement = self.account_achievements.get(id);
                let current = account_achievement.map(|aa| aa.current).flatten();
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

                ListItem::new(
                    percent_complete.map_or(
                        format!("       {}", achievement_name),
                        |percent_complete| {
                            format!("({:>3}%) {}", percent_complete, achievement_name)
                        },
                    ),
                )
            }
            Track::Custom(item) => ListItem::new(format!("       {}", item)),
        }
    }

    fn draw_achievement_info<B: tui::backend::Backend>(
        &mut self,
        achievement_id: usize,
        frame: &mut Frame<B>,
        area: Rect,
    ) {
        if let Some(achievement) = self.achievements.get(&achievement_id) {
            let account_achievement = self.account_achievements.get(&achievement.id);
            let progress_height = achievement.tiers.len() as u16 * self.tier_progress_bar_height;
            let info_chunks: Vec<Rect> = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(progress_height),
                    Constraint::Percentage(100),
                ])
                .split(area);

            self.draw_progress(frame, info_chunks[0], &achievement, account_achievement);
            self.draw_info(frame, info_chunks[1], &achievement, account_achievement)
        }
    }

    fn draw_sidebar<B: tui::backend::Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        frame.render_stateful_widget(
            List::new(
                self.tracks
                    .iter()
                    .map(|track| self.new_list_item(track))
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
    fn name(&self) -> &'static str {
        "Tracks"
    }

    fn draw(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let h_chunks: Vec<Rect> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(50), Constraint::Percentage(100)])
            .split(area);

        self.draw_sidebar(frame, h_chunks[0]);

        if let Some(track) = self.selected_track() {
            match track {
                Track::Achievement(id) => self.draw_achievement_info(id, frame, h_chunks[1]),
                Track::Custom(_) => {}
            }
        }

        self.add_track_popup.draw(frame, area);
    }

    fn handle_input(&mut self, event: &InputEvent) -> bool {
        if self.add_track_popup.handle_input(event) {
            return true;
        }

        match event.input {
            InputKind::MoveUp(amount) => {
                self.list_state.move_cursor(
                    self.app_state.tracked_items().len(),
                    CursorMovement::Up(amount),
                );
                true
            }
            InputKind::MoveDown(amount) => {
                self.list_state.move_cursor(
                    self.app_state.tracked_items().len(),
                    CursorMovement::Down(amount),
                );
                true
            }
            InputKind::Select => {
                if let Some(track) = self.selected_track() {
                    let _ = self.tx_event.send(Event::ToggleTrack(track));
                }
                true
            }
            InputKind::New => {
                self.add_track_popup.active(true);
                true
            }
            _ => false,
        }
    }

    fn handle_event(&mut self, event: &Event) {
        match event {
            Event::AchievementsLoaded(all_achievements) => {
                self.achievements = all_achievements
                    .into_iter()
                    .map(|achievement| (achievement.id, achievement.to_owned()))
                    .collect();
            }
            Event::AccountAchievementsLoaded(all_account_achievements) => {
                self.account_achievements = all_account_achievements
                    .0
                    .iter()
                    .map(|account_achievement| {
                        (account_achievement.id, account_achievement.to_owned())
                    })
                    .collect()
            }
            Event::ToggleTrack(_) | Event::AddTrack(_) => {
                self.tracks = self.app_state.tracked_items().into_iter().collect();
            }
            _ => {}
        }
    }
}
