use crate::{dto::Direction, model::Player};
use std::{
    collections::HashMap,
    sync::{atomic::AtomicUsize, Arc, Mutex},
};
use tokio::sync::broadcast;
use tracing::info;
use ulid::{Generator, Ulid};

#[derive(Clone)]
pub(crate) struct Repository {
    player_count: Arc<AtomicUsize>,
    state: Arc<Mutex<HashMap<Ulid, Player>>>,
    generator: Arc<Mutex<Generator>>,
    chat: Arc<Mutex<broadcast::Sender<String>>>,
}

impl Repository {
    pub(crate) fn initialize() -> Self {
        let player_count = Arc::new(AtomicUsize::new(0));
        let state = Arc::new(Mutex::new(HashMap::new()));
        let generator = Arc::new(Mutex::new(ulid::Generator::new()));
        let (sender, _) = broadcast::channel::<String>(100);
        let chat = Arc::new(Mutex::new(sender));

        Self {
            player_count,
            state,
            generator,
            chat,
        }
    }

    pub(crate) fn new_ulid(&self) -> Ulid {
        let mut generator = self.generator.lock().unwrap();
        loop {
            if let Ok(ulid) = generator.generate() {
                return ulid;
            }
            std::thread::yield_now();
        }
    }

    pub(crate) fn add_player(&self, name: &str) -> Ulid {
        let mut state = self.state.lock().unwrap();

        let ulid = self.new_ulid();
        let player = Player::new(ulid, name.to_owned());

        if state.insert(ulid, player).is_some() {
            unreachable!("ulid are unique");
        }
        self.player_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        info!(
            username = name,
            ulid = ulid.to_string(),
            "Registered player; total player count: {}",
            self.player_count.load(std::sync::atomic::Ordering::Relaxed)
        );
        ulid
    }

    pub(crate) fn get_chat(&self) -> (broadcast::Sender<String>, broadcast::Receiver<String>) {
        let sender = {
            let guard = self.chat.lock().unwrap();
            guard.clone()
        };

        (sender.clone(), sender.subscribe())
    }

    pub(crate) fn move_player(&self, key: Ulid, direction: &Direction) -> (i8, i8) {
        let mut state = self.state.lock().unwrap();

        let player = state.get_mut(&key).expect("connected players exist in map");

        player.move_player(direction);
        player.get_position()
    }

    pub(crate) fn get_nearby_players(&self, key: Ulid) -> Vec<(f64, Player)> {
        let state = self.state.lock().unwrap();

        let p1 = state.get(&key).unwrap().get_position();
        let nearby_players = state
            .values()
            .filter_map(|pair| {
                let p2 = pair.get_position();
                let dx = f64::from(p2.0) - f64::from(p1.0);
                let dy = f64::from(p2.1) - f64::from(p1.1);
                let distance = (dx * dx + dy * dy).sqrt();

                (distance <= 5.0).then(|| (distance, pair.clone()))
            })
            .collect::<Vec<_>>();
        nearby_players
    }

    pub(crate) fn get_player_position(&self, key: Ulid) -> (i8, i8) {
        let state = self.state.lock().unwrap();

        let Some(player) = state.get(&key) else {
            unreachable!("connected players exist in map")
        };

        player.get_position()
    }

    pub(crate) fn remove_player(&self, key: Ulid) {
        let mut state = self.state.lock().unwrap();

        let Some(player) = state.remove(&key) else {
            unreachable!("player disconnected, but it's record exist in map")
        };
        self.player_count
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);

        info!(
            username = player.name,
            ulid = player.ulid.to_string(),
            "Removed player; total player count: {}",
            self.player_count.load(std::sync::atomic::Ordering::Relaxed)
        );
    }
}
