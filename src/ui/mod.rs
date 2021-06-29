pub mod component;
mod view;
mod widget;

extern crate crossterm;
extern crate tui;

use std::{
    io::{self, Stdout},
    rc::Rc,
};

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use tokio::{
    select,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, Borders, Tabs},
    Terminal,
};

use crate::{
    events::{Event, ViewEvent},
    input::{Input, InputEvent, InputKind},
    state::AppState,
};

use self::view::{
    achievements::AchievementsView, dailies::DailiesView, status::StatusView, timer::TimerView,
    tracks::TracksView, View,
};

pub struct UI {
    app_state: Rc<AppState>,
    rx_view_event: UnboundedReceiver<ViewEvent>,
    status_view: StatusView,
    tracks_view: TracksView,
    achievements_view: AchievementsView,
    dailies_view: DailiesView,
    timer_view: TimerView,
    quit: bool,
}

impl UI {
    pub fn new(
        app_state: Rc<AppState>,
        tx_event: UnboundedSender<Event>,
        tx_view_event: UnboundedSender<ViewEvent>,
        rx_view_event: UnboundedReceiver<ViewEvent>,
    ) -> Self {
        let achievements_view = AchievementsView::new(app_state.clone(), tx_event.clone());
        let tracks_view = TracksView::new(app_state.clone(), tx_event);
        let status_view = StatusView::new(tx_view_event);
        let dailies_view = DailiesView::new(app_state.clone());
        let timer_view = TimerView::new();

        Self {
            app_state,
            rx_view_event,
            achievements_view,
            tracks_view,
            status_view,
            dailies_view,
            timer_view,
            quit: false,
        }
    }

    pub async fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Initial render
        terminal.clear()?;
        self.render(&mut terminal);

        enable_raw_mode()?;

        let (tx_input, mut rx_input) = mpsc::unbounded_channel::<InputEvent>();
        let input = Input::new(tx_input.clone());
        tokio::spawn(input.run());

        // Render loop
        loop {
            if self.quit {
                break;
            }
            select! {
                Some(input_event) = rx_input.recv() => {
                    self.handle_input_event(input_event);
                    self.render(&mut terminal);
                },
                Some(view_event) = self.rx_view_event.recv() => {
                    self.handle_view_event(view_event);
                    self.render(&mut terminal);
                }
            }
        }

        disable_raw_mode()?;
        Ok(())
    }

    pub fn handle_input_event(&mut self, input_event: InputEvent) {
        // Pass input events to current view
        if !match self.app_state.current_tab() {
            Some(0) => self.tracks_view.handle_input_event(&input_event),
            Some(1) => self.achievements_view.handle_input_event(&input_event),
            Some(2) => self.dailies_view.handle_input_event(&input_event),
            Some(3) => self.timer_view.handle_input_event(&input_event),
            _ => false,
        } {
            // If view doesn't consume input, handle it locally
            match input_event.input {
                InputKind::Quit => self.quit = true,
                InputKind::SwitchTab(tab_index) => self.app_state.select_tab(tab_index),
                _ => {}
            }
        }
    }

    pub fn handle_view_event(&mut self, view_event: ViewEvent) {
        if let ViewEvent::Quit = view_event {
            self.quit = true;
        }
        self.status_view.handle_view_event(&view_event);
        self.tracks_view.handle_view_event(&view_event);
        self.achievements_view.handle_view_event(&view_event);
        self.dailies_view.handle_view_event(&view_event);
        self.timer_view.handle_view_event(&view_event);
    }

    fn render(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) {
        let tabs = Tabs::new(vec![
            Spans::from("Tracks"),
            Spans::from("Achievements"),
            Spans::from("Dailies"),
            Spans::from("Timers"),
        ])
        .block(Block::default().borders(Borders::BOTTOM | Borders::TOP))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::White),
        )
        .style(Style::default().fg(Color::DarkGray))
        .select(self.app_state.current_tab().unwrap_or_default());

        let _ = terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Min(3),
                    Constraint::Length(frame.size().height - 5),
                    Constraint::Min(2),
                ])
                .split(frame.size());

            // Draw tabs
            frame.render_widget(tabs, chunks[0]);

            // Draw main center panel
            match self.app_state.current_tab() {
                Some(0) => self.tracks_view.draw(frame, chunks[1]),
                Some(1) => self.achievements_view.draw(frame, chunks[1]),
                Some(2) => self.dailies_view.draw(frame, chunks[1]),
                Some(3) => self.timer_view.draw(frame, chunks[1]),
                _ => {}
            }

            // Draw bottom status bar
            self.status_view.draw(frame, chunks[2])
        });
    }
}
