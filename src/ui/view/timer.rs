use std::{io::Stdout, iter, ops::Sub};

use chrono::{Duration, Timelike, Utc};
use gw2timers::{
    category::Category,
    event::EventInstance,
    meta::{MapMeta, MapMetaKind},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Cell, Row, Table},
    Frame,
};

use crate::{events::Event, input::InputEvent};

use super::View;

pub struct TimerView {
    heading_width: u16,
    event_width: u16,
}

impl TimerView {
    pub fn new() -> Self {
        Self {
            heading_width: 19,
            event_width: 40,
        }
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

    fn new_meta_row<'a>(
        meta: &MapMetaKind,
        time: Duration,
        name: &'static str,
        color: Color,
        mut num_events: u16,
    ) -> Row<'a> {
        let mut meta_iter = meta.into_iter().fast_forward(time);

        let heading = Cell::from(name).style(
            Style::default()
                .bg(Color::Black)
                .fg(color)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED),
        );

        let first_column = if let Some(event_instance) = meta_iter.now() {
            num_events -= 1;
            Self::new_event_cell(event_instance, color, time)
        } else {
            Self::new_event_cell(meta_iter.next().unwrap(), color, time)
        };

        let mut remaining_columns = meta_iter
            .take(num_events as usize)
            .map(|e| Self::new_event_cell(e, color, time))
            .collect::<Vec<Cell>>();
        remaining_columns.insert(0, heading);
        remaining_columns.insert(1, first_column);

        Row::new(remaining_columns)
    }
}

impl View for TimerView {
    fn name(&self) -> &'static str {
        "Timers"
    }

    fn draw(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let num_events = area.width / self.event_width;
        let event_area_width = area.width - self.heading_width;
        let mut constaints = iter::repeat(Constraint::Length(event_area_width / num_events))
            .take(num_events as usize)
            .collect::<Vec<Constraint>>();
        constaints.insert(0, Constraint::Length(18));

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
                            meta.name,
                            Self::meta_color(meta),
                            num_events,
                        )
                    })
                    .collect::<Vec<Row>>(),
            )
            .widths(&constaints),
            area,
        );
    }

    fn handle_input(&mut self, _: &InputEvent) -> bool {
        false
    }

    fn handle_event(&mut self, _: &Event) {}
}
