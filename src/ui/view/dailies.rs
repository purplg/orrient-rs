use std::{collections::HashMap, rc::Rc};

use tui::{
    backend::Backend,
    layout::Rect,
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::Paragraph,
    Frame,
};

use crate::{
    api::{Achievement, Dailies, Daily},
    events::ViewEvent,
    input::InputEvent,
    state::AppState,
};

use super::View;

pub struct DailiesView {
    app_state: Rc<AppState>,
    achievements: HashMap<usize, Achievement>,
    dailies: Option<Dailies>,
    header_style: Style,
}

impl DailiesView {
    pub fn new(app_state: Rc<AppState>) -> Self {
        DailiesView {
            app_state,
            achievements: HashMap::default(),
            dailies: None,
            header_style: Style::default().add_modifier(Modifier::BOLD),
        }
    }

    fn render_category(&self, title: String, dailies: &[Daily]) -> Vec<Spans> {
        if !dailies.is_empty() {
            let mut group = vec![Spans::from(Span::styled(title, self.header_style))];
            for a in dailies.iter().filter_map(|daily| self.render_daily(daily)) {
                group.push(a)
            }
            group
        } else {
            Vec::default()
        }
    }

    fn render_daily(&self, daily: &Daily) -> Option<Spans> {
        self.achievements.get(&daily.id).map(|achievement| {
            Spans::from(vec![
                Span::raw(format!("{}: ", achievement.name.clone())),
                Span::styled(
                    achievement.requirement.clone(),
                    Style::default().add_modifier(Modifier::DIM),
                ),
            ])
        })
    }
}

impl View for DailiesView {
    fn draw<B: Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        if let Some(dailies) = &self.dailies {
            let blank = vec![Spans::default()];
            let pve = self.render_category("PvE".to_string(), &dailies.pve);
            let pvp = self.render_category("PvP".to_string(), &dailies.pvp);
            let wvw = self.render_category("WvW".to_string(), &dailies.wvw);
            let fractals = self.render_category("Fractals".to_string(), &dailies.fractals);
            let special = self.render_category("Special".to_string(), &dailies.special);

            let widget = Paragraph::new(
                pve.into_iter()
                    .chain(blank.iter().map(ToOwned::to_owned))
                    .chain(pvp.into_iter())
                    .chain(blank.iter().map(ToOwned::to_owned))
                    .chain(wvw.into_iter())
                    .chain(blank.iter().map(ToOwned::to_owned))
                    .chain(fractals.into_iter())
                    .chain(blank.iter().map(ToOwned::to_owned))
                    .chain(special.into_iter())
                    .collect::<Vec<Spans>>(),
            );
            frame.render_widget(widget, area);
        }
    }

    fn handle_input_event(&mut self, _: &InputEvent) -> bool {
        false
    }

    fn handle_view_event(&mut self, view_event: &ViewEvent) {
        match view_event {
            ViewEvent::UpdateAchievements => {
                self.achievements = self.app_state.achievements();
            }
            ViewEvent::UpdateDailies => {
                self.dailies = self.app_state.dailies();
            }
            _ => {}
        }
    }
}
