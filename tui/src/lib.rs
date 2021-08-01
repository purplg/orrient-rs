pub mod component;
mod input;
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

use crate::input::{Input, InputEvent, InputKind};
use orrient::{config::Config, events::Event, state::AppState};

use self::view::{
    achievements::AchievementsView, bookmarks::BookmarksView, dailies::DailiesView,
    status::StatusView, timer::TimerView, tracks::TracksView, View,
};

pub struct UI {
    app_state: Rc<AppState>,
    rx_event: UnboundedReceiver<Event>,
    tabs: Vec<Box<dyn View>>,
    tab_names: Vec<&'static str>,
    status_view: StatusView,
    quit: bool,
    current_tab: usize,
}

impl UI {
    pub fn new(
        config: &Config,
        app_state: Rc<AppState>,
        tx_event: UnboundedSender<Event>,
        rx_event: UnboundedReceiver<Event>,
    ) -> Self {
        let tabs = vec![
            Box::new(TracksView::new(app_state.clone(), tx_event.clone())) as Box<dyn View>,
            Box::new(AchievementsView::new(app_state.clone(), tx_event.clone())),
            Box::new(DailiesView::new()),
            Box::new(TimerView::new()),
            Box::new(BookmarksView::new(app_state.clone(), tx_event.clone())),
        ];
        let tab_names = tabs.iter().map(|tab| tab.name()).collect::<Vec<_>>();

        let status_view = StatusView::new(tx_event);

        Self {
            app_state,
            rx_event,
            tabs,
            tab_names,
            status_view,
            quit: false,
            current_tab: config.starting_tab - 1,
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
                    self.handle_input(input_event);
                    self.render(&mut terminal);
                },
                Some(view_event) = self.rx_event.recv() => {
                    self.handle_event(view_event);
                    self.render(&mut terminal);
                }
            }
        }

        disable_raw_mode()?;
        Ok(())
    }

    fn select_tab(&mut self, tab_index: usize) {
        if tab_index < self.tabs.len() {
            self.current_tab = tab_index;
        }
    }

    pub fn handle_input(&mut self, input_event: InputEvent) {
        // Pass input events to current view
        if !self
            .tabs
            .get_mut(self.current_tab)
            .map_or(false, |tab| tab.handle_input(&input_event))
        {
            // If view doesn't handle input, handle it locally
            match input_event.input {
                InputKind::Quit => self.quit = true,
                InputKind::SwitchTab(tab_index) => self.select_tab(tab_index),
                _ => {}
            }
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        match &event {
            Event::Quit => self.quit = true,
            Event::ToggleTrack(track) => self.app_state.toggle_track(track),
            _ => {}
        }
        self.status_view.handle_event(&event);
        for tab in self.tabs.iter_mut() {
            tab.handle_event(&event);
        }
    }

    fn render(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) {
        let tabs = Tabs::new(self.tab_names.iter().map(|s| Spans::from(*s)).collect())
            .block(Block::default().borders(Borders::BOTTOM | Borders::TOP))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::White),
            )
            .style(Style::default().fg(Color::DarkGray))
            .select(self.current_tab);

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

            // Draw current tab
            if let Some(tab) = self.tabs.get_mut(self.current_tab) {
                tab.draw(frame, chunks[1]);
            }

            // Draw bottom status bar
            self.status_view.draw(frame, chunks[2])
        });
    }
}
