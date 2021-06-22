use crate::ui::widget::text_box::TextBox;
use std::{
    collections::{BTreeMap, HashMap},
    rc::Rc,
};

use crossterm::event::KeyCode;
use tokio::sync::mpsc::UnboundedSender;
use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::{
    api::{AccountAchievement, Achievement},
    events::{CursorMovement, Event, InputEvent, InputKind, StateEvent, ViewEvent},
    state::AppState,
    tracks::Track,
    ui::{component::achievement_info::AchievementInfo, widget::list_selection::ListSelection},
};

use super::View;

struct AchievementStatusStyles {
    pub normal: Style,
    pub done: Style,
    pub daily: Style,
    pub unknown_progress: Style,
    pub tracked: Style,
}

pub struct AchievementsView {
    app_state: Rc<AppState>,
    list_state: ListState,
    achievements: BTreeMap<usize, Achievement>,
    account_achievements: HashMap<usize, AccountAchievement>,
    tx_state: UnboundedSender<Event>,
    visible_list_ids: Vec<usize>,
    searching: bool,
    search_string: String,
    style: AchievementStatusStyles,
}

impl AchievementsView {
    pub fn new(app_state: Rc<AppState>, tx_state: UnboundedSender<Event>) -> Self {
        AchievementsView {
            app_state,
            tx_state,
            list_state: ListState::default(),
            achievements: BTreeMap::default(),
            account_achievements: HashMap::default(),
            visible_list_ids: Vec::default(),
            searching: false,
            search_string: String::default(),
            style: AchievementStatusStyles {
                normal: Style::default(),
                done: Style::default().fg(Color::Green),
                daily: Style::default().fg(Color::Blue),
                unknown_progress: Style::default().fg(Color::Red),
                tracked: Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD),
            },
        }
    }

    fn new_list_item<'a>(
        achievement: &Achievement,
        account_achievement: Option<&AccountAchievement>,
        tracked: bool,
        styles: &AchievementStatusStyles,
    ) -> ListItem<'a> {
        let mut style = if let Some(account_achievement) = account_achievement {
            if account_achievement.done {
                styles.done
            } else {
                styles.normal
            }
        } else if achievement.flags.contains(&"Daily".to_string()) {
            styles.daily
        } else {
            styles.unknown_progress
        };

        if tracked {
            style = style.patch(styles.tracked);
        }

        let text = Text::styled(achievement.name.clone(), style);
        ListItem::new(text)
    }

    fn selected_id(&self) -> Option<usize> {
        if let Some(selected_index) = self.list_state.selected() {
            self.visible_list_ids
                .get(selected_index)
                .map(ToOwned::to_owned)
        } else {
            None
        }
    }

    fn update_filter(&mut self) {
        self.visible_list_ids = self
            .achievements
            .iter()
            .filter_map(|(id, achievement)| {
                if achievement
                    .name
                    .to_lowercase()
                    .contains(&self.search_string.to_lowercase())
                {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect::<Vec<usize>>();

        // HACK Since the ListState offset can cause an 'index out of bounds' panic, we have to select nothing to reset the ListState and then re-apply cursor position
        let before_pos = self.list_state.selected().unwrap_or(0);
        self.list_state.select(None);
        self.list_state.move_cursor(
            self.visible_list_ids.len(),
            CursorMovement::Down(before_pos),
        );
    }
}

impl View for AchievementsView {
    fn draw<B: tui::backend::Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        // LAYOUTS
        let horiz_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(50), Constraint::Percentage(100)])
            .split(area);

        let (main_panel, list_panel, search_panel) =
            if !self.searching && self.search_string.is_empty() {
                let left_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(100)])
                    .split(horiz_layout[0]);

                (horiz_layout[1], left_layout[0], None)
            } else {
                let left_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(2), Constraint::Percentage(100)])
                    .split(horiz_layout[0]);

                (horiz_layout[1], left_layout[1], Some(left_layout[0]))
            };

        // ACHIEVEMENT LIST
        let list_items = self
            .visible_list_ids
            .iter()
            .filter_map(|id| self.achievements.get(id))
            .map(|achievement| {
                let account_achievement = self.account_achievements.get(&achievement.id);
                Self::new_list_item(
                    achievement,
                    account_achievement,
                    self.app_state
                        .is_tracked(Track::Achievement(achievement.id)),
                    &self.style,
                )
            })
            .collect::<Vec<ListItem>>();

        // RENDER
        // achievement list in the left sidebar
        frame.render_stateful_widget(
            List::new(list_items)
                .block(Block::default().borders(Borders::RIGHT))
                .highlight_symbol(">>"),
            list_panel,
            &mut self.list_state,
        );

        // Render the main selected achievement information in the right panel
        if let Some(achievement) = self
            .selected_id()
            .map(|achievement_id| self.achievements.get(&achievement_id))
            .flatten()
        {
            let account_achievement = self.account_achievements.get(&achievement.id);

            frame.render_widget(
                AchievementInfo::new_widget(achievement, account_achievement),
                main_panel,
            );
        }

        // Render the bottom bar to display filter search
        if let Some(bottom_panel_chunk) = search_panel {
            frame.render_widget(
                TextBox::new(&self.search_string)
                    .block(Block::default().borders(Borders::BOTTOM | Borders::RIGHT))
                    .style(if self.searching {
                        Style::default()
                    } else {
                        Style::default().add_modifier(Modifier::DIM)
                    }),
                bottom_panel_chunk,
            );
        }
    }

    fn handle_input_event(&mut self, event: &InputEvent) -> bool {
        if self.searching {
            if let Some(key_code) = event.key_code {
                match key_code {
                    KeyCode::Enter => {
                        self.searching = false;
                        true
                    }
                    KeyCode::Esc => {
                        self.search_string = "".to_string();
                        self.update_filter();
                        self.searching = false;
                        true
                    }
                    KeyCode::Char(letter) => {
                        self.search_string.push(letter);
                        self.update_filter();
                        true
                    }
                    KeyCode::Backspace => {
                        self.search_string.pop();
                        self.update_filter();
                        true
                    }
                    KeyCode::Left
                    | KeyCode::Right
                    | KeyCode::Home
                    | KeyCode::End
                    | KeyCode::Delete => false,
                    _ => false,
                }
            } else {
                false
            }
        } else {
            match event.input {
                InputKind::MoveUp(amount) => {
                    self.list_state
                        .move_cursor(self.visible_list_ids.len(), CursorMovement::Up(amount));
                    true
                }
                InputKind::MoveDown(amount) => {
                    self.list_state
                        .move_cursor(self.visible_list_ids.len(), CursorMovement::Down(amount));
                    true
                }
                InputKind::Search => {
                    self.searching = !self.searching;
                    true
                }
                InputKind::Top => {
                    if self.visible_list_ids.is_empty() {
                        self.list_state.select(Some(0));
                    }
                    true
                }
                InputKind::Bottom => {
                    self.list_state
                        .select(Some(self.visible_list_ids.len() - 1));
                    true
                }
                InputKind::Back => {
                    self.search_string = "".to_string();
                    self.update_filter();
                    true
                }
                InputKind::Track => {
                    self.selected_id().map(|id| {
                        self.achievements.get(&id).map(|achievement| {
                            self.tx_state.send(Event::State(StateEvent::ToggleTrack(
                                Track::Achievement(achievement.id),
                            )))
                        })
                    });
                    true
                }
                _ => false,
            }
        }
    }

    fn handle_view_event(&mut self, event: &ViewEvent) {
        match event {
            ViewEvent::UpdateAchievements => {
                self.achievements = self
                    .app_state
                    .achievements()
                    .into_iter()
                    .collect::<BTreeMap<usize, Achievement>>();
                self.update_filter()
            }
            &ViewEvent::UpdateAccountAchievements => {
                self.account_achievements = self.app_state.account_achievements()
            }
            _ => {}
        };
    }
}
