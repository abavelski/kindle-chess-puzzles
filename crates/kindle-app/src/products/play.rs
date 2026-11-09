//! Play-only composition. All socket work belongs to one detached worker;
//! shutdown never waits for a cold Maia request and there are no save-game writes.
use crate::{
    play_http::{Endpoint, HttpClient, PlayClient},
    runtime::{Application, Poll},
};
use chess_core::play::{PlayApp, Request, Response};
use chess_render::{HitTarget, PlayTarget, ScreenState};
use std::{
    io::Read,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, SyncSender, TryRecvError},
};
struct Worker {
    requests: SyncSender<(u64, Request)>,
    responses: Receiver<(u64, Result<Response, String>)>,
}
impl Worker {
    fn start(client: impl PlayClient) -> Self {
        let (requests, work) = mpsc::sync_channel::<(u64, Request)>(1);
        let (done, responses) = mpsc::sync_channel(1);
        std::thread::spawn(move || {
            while let Ok((id, request)) = work.recv() {
                let result = client.execute(&request);
                if done.send((id, result)).is_err() {
                    break;
                }
            }
        });
        Self {
            requests,
            responses,
        }
    }
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Config {
    server_url: String,
}
fn load_config(root: &Path) -> Result<Endpoint, String> {
    let file = std::fs::File::open(root.join("config.json")).map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    file.take(4097)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 4096 {
        return Err("config.json too large".into());
    }
    let config: Config = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    Endpoint::parse(&config.server_url)
}
pub struct PlayRuntime {
    state: PlayApp,
    worker: Option<Worker>,
    root: Option<PathBuf>,
}
impl PlayRuntime {
    pub fn open(root: PathBuf) -> Self {
        let mut runtime = Self {
            state: PlayApp::default(),
            worker: None,
            root: Some(root),
        };
        runtime.reconnect();
        runtime
    }
    pub fn with_client(client: impl PlayClient) -> Self {
        let mut runtime = Self {
            state: PlayApp::default(),
            worker: Some(Worker::start(client)),
            root: None,
        };
        runtime.request(Request::Health);
        runtime
    }
    pub fn state(&self) -> &PlayApp {
        &self.state
    }
    fn reconnect(&mut self) {
        if self.state.in_flight() {
            return;
        }
        if let Some(root) = &self.root {
            match load_config(root) {
                Ok(endpoint) => self.worker = Some(Worker::start(HttpClient::new(endpoint))),
                Err(error) => {
                    self.worker = None;
                    if let Some(id) = self.state.begin(Request::Current) {
                        self.state.complete(
                            id,
                            Err(format!(
                                "Configure serverUrl in {}/config.json: {error}",
                                root.display()
                            )),
                        );
                    }
                    return;
                }
            }
        }
        self.request(Request::Health);
    }
    fn request(&mut self, request: Request) {
        let Some(worker) = &self.worker else {
            return;
        };
        let Some(id) = self.state.begin(request.clone()) else {
            return;
        };
        if worker.requests.try_send((id, request)).is_err() {
            self.state
                .complete(id, Err("HTTP worker unavailable".into()));
        }
    }
}
impl Application for PlayRuntime {
    fn screen(&self) -> &impl ScreenState {
        &self.state
    }
    fn poll(&mut self) -> Poll {
        let Some(worker) = &self.worker else {
            return Poll::Idle;
        };
        match worker.responses.try_recv() {
            Ok((id, response)) => {
                if let Some(next) = self.state.complete(id, response) {
                    self.request(next);
                }
                Poll::Changed
            }
            Err(TryRecvError::Empty) => Poll::Idle,
            Err(TryRecvError::Disconnected) => {
                self.worker = None;
                // Surface a failed worker without replaying an ambiguous request.
                // Pending completions cannot be replaced by a new request.
                self.state.worker_failed();
                Poll::Changed
            }
        }
    }
    fn tap(&mut self, target: HitTarget) -> bool {
        let HitTarget::Play(target) = target else {
            return false;
        };
        let request = match target {
            PlayTarget::Close => return true,
            PlayTarget::Square(square) => self.state.tap(square),
            PlayTarget::Promotion(kind) => self.state.promote(kind),
            PlayTarget::CancelPromotion => {
                self.state.cancel_selection();
                None
            }
            PlayTarget::RatingDown => {
                self.state.change_rating(-100);
                None
            }
            PlayTarget::RatingUp => {
                self.state.change_rating(100);
                None
            }
            PlayTarget::NewGame => Some(Request::New {
                rating: self.state.rating,
            }),
            PlayTarget::Resign => Some(Request::Resign),
            PlayTarget::Retry => {
                self.reconnect();
                None
            }
            PlayTarget::FullRefresh => None,
        };
        if let Some(request) = request {
            self.request(request);
        }
        false
    }
    fn pause_input(&mut self) {
        self.state.cancel_selection();
    }
    fn flush(&mut self) {}
}
