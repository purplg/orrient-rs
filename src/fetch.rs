use std::{sync::Arc, time::Duration};

use log::debug;
use tokio::sync::mpsc::UnboundedSender;

use crate::{api::{Achievement, AllAccountAchievements, AllAchievementIDs, Dailies}, client::CachedClient, events::{Event, StateEvent, ViewEvent}};

pub struct Fetch {
    client: Arc<CachedClient>,
    tx_state: UnboundedSender<Event>,
    all_achievement_ids: Vec<usize>,
}

impl Fetch {
    pub fn new(client: Arc<CachedClient>, tx_state: UnboundedSender<Event>) -> Fetch {
        Fetch {
            client,
            tx_state,
            all_achievement_ids: Vec::default(),
        }
    }

    pub async fn run(mut self, fetch_tick: u64) {
        // TODO Error handling
        // Fetch all the existing achievement IDs
        self.all_achievement_ids = match self.client.request::<AllAchievementIDs>().await {
            Ok(achievement_ids) => Some(achievement_ids.0),
            Err(_) => None,
        }
        .unwrap_or_default();

        self.fetch_achievements().await;
        self.fetch_dailies().await;
        self.loop_fetch_account_achievements(fetch_tick).await;
    }

    // Requests and caches all the achievements in the game
    async fn fetch_achievements(&self) {
        let paged_ids = self.all_achievement_ids.chunks(100);
        let total_pages = paged_ids.clone().count();
        for (current_page, ids) in paged_ids.enumerate() {
            match self
                .client
                .request_many::<Achievement, usize>(&ids.to_vec())
                .await
            {
                Ok(achievements) => {
                    let progress: f64 = current_page as f64 / (total_pages - 1) as f64;
                    let _ =
                        self.tx_state
                            .send(Event::State(StateEvent::FetchedAchievements {
                                achievements,
                            }));
                    let _ = self
                        .tx_state
                        .send(Event::View(ViewEvent::UpdateStatus(format!(
                            "Loading achievements... {}%",
                            (progress * 100.0) as u64
                        ))));
                }
                Err(err) => {
                    debug!("Error fetching Achievements: {:?}", err);
                }
            }
        }
        let _ = self
            .tx_state
            .send(Event::State(StateEvent::AchievementsLoaded));
        let _ = self
            .tx_state
            .send(Event::View(ViewEvent::UpdateStatus(
                "Done loading achievements...".to_string(),
            )));
        self.client.write_cache();
    }

    // A loop to periodically update account achievement progress
    async fn loop_fetch_account_achievements(&self, tick: u64) {
        loop {
            self.fetch_account_achievements().await;
            tokio::time::sleep(Duration::from_secs(tick)).await;
        }
    }

    // Update account achievement status
    async fn fetch_account_achievements(&self) {
        match self.client.request::<AllAccountAchievements>().await {
            Ok(all_account_achievements) => {
                let _ = self.tx_state.send(Event::State(
                    StateEvent::FetchedAccountAchievements {
                        all_account_achievements,
                    },
                ));
                let _ = self
                    .tx_state
                    .send(Event::View(ViewEvent::UpdateStatus(
                        "Updated achievement progress".to_string(),
                    )));
            }
            Err(err) => debug!("Error fetching AllAccountAchievements: {:?}", err),
        }
        self.client.write_cache();
    }

    async fn fetch_dailies(&self) {
        match self.client.request::<Dailies>().await {
            Ok(dailies) => {
                let _ = self
                    .tx_state
                    .send(Event::State(StateEvent::FetchedDailies(dailies)));
            }
            Err(err) => debug!("Error fetching Dailies: {:?}", err),
        }
    }
}
