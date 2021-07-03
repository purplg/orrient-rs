use std::{collections::HashSet, rc::Rc};

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{
    api::{Achievement, AllAccountAchievements, Dailies},
    state::AppState,
    tracks::Track,
};

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
    AccountAchievementsLoaded(AllAccountAchievements),
    AchievementsLoaded(HashSet<Achievement>),
    FetchedDailies(Dailies),
}

#[derive(Debug)]
pub enum ViewEvent {
    UpdateTracks,
    UpdateAchievements(HashSet<Achievement>),
    UpdateAccountAchievements(AllAccountAchievements),
    UpdateDailies(Dailies),
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
                    let _ = self.tx_event.send(Event::View(ViewEvent::UpdateTracks));
                }
                StateEvent::ToggleTrack(track) => {
                    self.app_state.toggle_track(track);
                    let _ = self.tx_event.send(Event::View(ViewEvent::UpdateTracks));
                }
                StateEvent::AchievementsLoaded(all_achievements) => {
                    let _ = self
                        .tx_event
                        .send(Event::View(ViewEvent::UpdateAchievements(all_achievements)));
                }
                StateEvent::AccountAchievementsLoaded(all_account_achievements) => {
                    let _ = self
                        .tx_event
                        .send(Event::View(ViewEvent::UpdateAccountAchievements(all_account_achievements)));
                }
                StateEvent::FetchedDailies(dailies) => {
                    let _ = self.tx_event.send(Event::View(ViewEvent::UpdateDailies(dailies)));
                }
            },
        }
    }
}
