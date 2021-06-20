use std::ops::Sub;

use chrono::{Duration, Timelike, Utc};
use gw2timers::{
    category::Category,
    event::EventInstance,
    meta::{MapMeta, MapMetaKind},
};
use tui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use crate::events::{InputEvent, ViewEvent};

use super::View;

pub struct TimerView;

impl TimerView {
    pub fn new() -> Self {
        Self
    }

    fn meta_color(meta: MapMeta) -> Color {
        match meta.category {
            Category::CoreTyria => Color::LightBlue,
            Category::LivingWorldSeason2 => Color::Yellow,
            Category::LivingWorldSeason3 => Color::Red,
            Category::LivingWorldSeason4 => Color::Magenta,
            Category::HeartOfThorns => Color::Green,
            Category::PathOfFire => Color::LightYellow,
            Category::TheIcebroodSaga => Color::Blue,
        }
    }

    fn time_until_fmt(time_until: Duration) -> String {
        if time_until.num_seconds() < 0 {
            format!("{:>4}m", time_until.num_minutes(),)
        } else {
            format!(
                "{:02}:{:02}",
                time_until.num_hours(),
                time_until.num_minutes() % 60,
            )
        }
    }

    fn new_event_cell<'a>(
        event_instance: EventInstance,
        color: Color,
        current_time_offset: Duration,
    ) -> Cell<'a> {
        let time_until_event = event_instance.start_time.sub(current_time_offset);
        Cell::from(format!(
            "{} - {}",
            Self::time_until_fmt(time_until_event),
            event_instance.schedule.name
        ))
        .style(Style::default().fg(color))
    }

    fn new_meta_row<'a>(meta: &MapMetaKind, time: Duration, name: String, color: Color) -> Row<'a> {
        let mut meta_iter = meta.into_iter().fast_forward(time);

        Row::new([
            // Place the name of the Meta map in the first column
            Cell::from(name).style(
                Style::default()
                    .bg(Color::Black)
                    .fg(color)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED),
            ),
            // If there's an event happening now, display that instead of the next event
            if let Some(event_instance) = meta_iter.now() {
                Self::new_event_cell(event_instance, color, time)
            } else {
                Self::new_event_cell(meta_iter.next().unwrap(), color, time)
            },
            Self::new_event_cell(meta_iter.next().unwrap(), color, time),
            Self::new_event_cell(meta_iter.next().unwrap(), color, time),
            Self::new_event_cell(meta_iter.next().unwrap(), color, time),
            Self::new_event_cell(meta_iter.next().unwrap(), color, time),
            Self::new_event_cell(meta_iter.next().unwrap(), color, time),
        ])
    }
}

impl View for TimerView {
    fn draw<B: tui::backend::Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        let current_time = Utc::now().time();
        let time = Duration::seconds(current_time.num_seconds_from_midnight() as i64);
        frame.render_widget(
            Table::new(
                MapMetaKind::all_keys()
                    .iter()
                    .map(|meta_key| {
                        let meta = meta_key.info();
                        Self::new_meta_row(
                            meta_key,
                            time,
                            meta.name.clone(),
                            Self::meta_color(meta),
                        )
                    })
                    .collect::<Vec<Row>>(),
            )
            .widths(&[
                Constraint::Percentage(10),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
            ]),
            area,
        );
    }

    fn handle_input_event(&mut self, _: &InputEvent) -> bool {
        false
    }

    fn handle_view_event(&mut self, _: &ViewEvent) {}
}
