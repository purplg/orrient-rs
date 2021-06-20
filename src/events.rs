use std::rc::Rc;

use crossterm::event::KeyCode;
use log::error;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{
    api::{Achievement, AllAccountAchievements, Dailies},
    state::AppState,
    tracks::{Track, Writer},
};

#[derive(Debug)]
pub enum Event {
    View(ViewEvent),
    State(StateEvent),
}

#[derive(Debug)]
pub enum StateEvent {
    Quit,
    LoadTracks(Vec<Track>),
    ToggleTrack(Track),
    FetchedAchievements {
        achievements: Vec<Achievement>,
    },
    FetchedAccountAchievements {
        all_account_achievements: AllAccountAchievements,
    },
    AchievementsLoaded,
    UpdateStatus(String),
    FetchedDailies(Dailies),
}

pub struct InputEvent {
    pub input: InputKind,
    pub key_code: Option<KeyCode>,
}

#[derive(Debug)]
pub enum InputKind {
    MoveUp(usize),
    MoveDown(usize),
    Top,
    Bottom,
    Back,
    Quit,
    Track,
    Search,
    SwitchTab(usize),
    Unhandled,
}

#[derive(Debug)]
pub enum ViewEvent {
    UpdateTracks,
    UpdateAchievements,
    UpdateAccountAchievements,
    UpdateDailies,
    Quit,
}

#[derive(Debug)]
pub enum CursorMovement {
    Up(usize),
    Down(usize),
    None,
}

pub struct EventLoop {
    app_state: Rc<AppState>,
    tx_event: UnboundedSender<Event>,
    rx_event: UnboundedReceiver<Event>,
    tx_view: UnboundedSender<ViewEvent>,
    tracks_writer: Writer<Vec<Track>>,
}

impl EventLoop {
    pub fn new(
        app_state: Rc<AppState>,
        tx_event: UnboundedSender<Event>,
        rx_event: UnboundedReceiver<Event>,
        tx_view: UnboundedSender<ViewEvent>,
        tracks_writer: Writer<Vec<Track>>,
    ) -> Self {
        Self {
            app_state,
            tx_event,
            rx_event,
            tx_view,
            tracks_writer,
        }
    }

    pub async fn run(mut self) {
        while let Some(event) = self.rx_event.recv().await {
            self.handle_event(event);
        }
    }

    pub fn handle_event(&self, event: Event) {
        match event {
            Event::View(view_event) => {
                let _ = self.tx_view.send(view_event);
            }
            Event::State(state_event) => match state_event {
                StateEvent::Quit => { let _ = self.tx_view.send(ViewEvent::Quit); },
                StateEvent::LoadTracks(tracks) => {
                    for track in tracks {
                        self.app_state.add_track(track);
                    }
                    let _ = self.tx_event.send(Event::View(ViewEvent::UpdateTracks));
                }
                StateEvent::ToggleTrack(track) => {
                    self.app_state.toggle_track(track);
                    let tracks = self
                        .app_state
                        .tracked_achievements()
                        .iter()
                        .map(|id| Track::Achievement(*id))
                        .collect::<Vec<Track>>();
                    if let Err(err) = self.tracks_writer.write(tracks) {
                        error!("Error writing tracks: {}", err);
                    }
                    let _ = self.tx_event.send(Event::View(ViewEvent::UpdateTracks));
                }
                StateEvent::FetchedAchievements { achievements } => {
                    self.app_state.insert_achievements(achievements);
                }
                StateEvent::AchievementsLoaded => {
                    let _ = self
                        .tx_event
                        .send(Event::View(ViewEvent::UpdateAchievements));
                }
                StateEvent::FetchedAccountAchievements {
                    all_account_achievements,
                } => {
                    self.app_state
                        .set_account_achievements(all_account_achievements);
                    let _ = self
                        .tx_event
                        .send(Event::View(ViewEvent::UpdateAccountAchievements));
                }
                StateEvent::FetchedDailies(dailies) => {
                    self.app_state.set_dailies(dailies);
                    let _ = self.tx_event.send(Event::View(ViewEvent::UpdateDailies));
                }
                StateEvent::UpdateStatus(message) => self.app_state.set_status(message),
            },
        }
    }
}
