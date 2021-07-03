use std::{collections::HashSet, rc::Rc};

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{api::{Achievement, AllAccountAchievements, Dailies}, state::AppState, tracks::Track};

#[derive(Debug)]
pub enum Event {
    View(ViewEvent),
    State(StateEvent),
}

#[derive(Debug)]
pub enum StateEvent {
    Quit,
    LoadTracks(HashSet<Track>),
    AddTrack(Track),
    ToggleTrack(Track),
    FetchedAchievements {
        achievements: Vec<Achievement>,
    },
    FetchedAccountAchievements {
        all_account_achievements: AllAccountAchievements,
    },
    AchievementsLoaded,
    FetchedDailies(Dailies),
}

#[derive(Debug)]
pub enum ViewEvent {
    UpdateTracks,
    UpdateAchievements,
    UpdateAccountAchievements,
    UpdateDailies,
    UpdateStatus(String),
    Quit,
}

pub struct EventLoop {
    app_state: Rc<AppState>,
    tx_event: UnboundedSender<Event>,
    rx_event: UnboundedReceiver<Event>,
    tx_view: UnboundedSender<ViewEvent>,
}

impl EventLoop {
    pub fn new(
        app_state: Rc<AppState>,
        tx_event: UnboundedSender<Event>,
        rx_event: UnboundedReceiver<Event>,
        tx_view: UnboundedSender<ViewEvent>,
    ) -> Self {
        Self {
            app_state,
            tx_event,
            rx_event,
            tx_view,
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
                StateEvent::Quit => {
                    let _ = self.tx_view.send(ViewEvent::Quit);
                }
                StateEvent::LoadTracks(tracks) => {
                    for track in tracks {
                        self.app_state.add_track(track);
                    }
                    let _ = self.tx_event.send(Event::View(ViewEvent::UpdateTracks));
                }
                StateEvent::AddTrack(track) => {
                    self.app_state.add_track(track);
                    // TODO save app state
                    // if let Err(err) = self.app_state.write(tracks) {
                    //     error!("Error writing tracks: {}", err);
                    // }
                    let _ = self.tx_event.send(Event::View(ViewEvent::UpdateTracks));
                }
                StateEvent::ToggleTrack(track) => {
                    self.app_state.toggle_track(track);
                    // TODO save app state
                    // if let Err(err) = self.tracks_writer.write(tracks) {
                    //     error!("Error writing tracks: {}", err);
                    // }
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
            },
        }
    }
}
