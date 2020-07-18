use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::{mpsc, RwLock};
use crate::hub::{Hub, User};
use warp::Filter;
use log::{error, info};
use warp::ws::WebSocket;
use futures::{FutureExt, StreamExt};
use crate::proto::{Output, KnowThatOutput};
use corr_core::runtime::Environment;
use app_dirs2::*;
use std::fs::File;
use corr_journeys::{JourneyStore, Interactable};

/// Our global unique user id counter.
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);
const APP_INFO: AppInfo = AppInfo{name: "corrs", author: "Atmaram Naik"};
pub struct Server{
    port: u16,
    hub:Arc<Hub>
}
impl Server {
    pub fn new(port: u16) -> Self {
        Server {
            port,
            hub:Arc::new(Hub::new())
        }
    }
    pub async fn run(&self) {
        let hub = self.hub.clone();

        let runner = warp::path("runner")
            .and(warp::ws())
            .and(warp::any().map(move || hub.clone()))
            .map(
                move |ws: warp::ws::Ws,
                      hub: Arc<Hub>| {
                        ws
                        .on_upgrade(move |web_socket| async move {
                            tokio::spawn(Self::user_connected(hub,web_socket));
                        })
                },
            );

        let shutdown = async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install CTRL+C signal handler");
        };
        let (_, serving) =
            warp::serve(runner).bind_with_graceful_shutdown(([127, 0, 0, 1], self.port), shutdown);


        tokio::select! {
            _ = serving => {}
        }
    }
    async fn user_connected(
        hub: Arc<Hub>,
        web_socket: WebSocket,
    ) {
        let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

        eprintln!("User Connected: {}", my_id);

        // Split the socket into a sender and receive of messages.
        let (user_ws_tx, mut user_ws_rx) = web_socket.split();
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::task::spawn(rx.forward(user_ws_tx).map(|result| {
            if let Err(e) = result {
                eprintln!("websocket send error: {}", e);
            }
        }));
        let mut user = User::new(tx,user_ws_rx);
        user.send(Output::KnowThat(KnowThatOutput{phrase:format!("Hello")}));
        tokio::spawn(async move {
            start(user);
        });
        // user.send(Output::KnowThat(KnowThatOutput{phrase:format!("Hello")}));
        // hub.users.write().await.insert(my_id, user);
    }
}
pub fn start(user:User){
    let mut journeys=Vec::new();
    let mut app_path=app_root(AppDataType::UserConfig, &APP_INFO).unwrap();
    let path=app_path.join("journeys");
    println!("{:?}",path);
    for dir_entry in std::fs::read_dir(path).unwrap(){
        let dir_entry=dir_entry.unwrap().path();
        if dir_entry.is_file() {
            if let Some(extention) = dir_entry.extension() {
                match extention.to_str() {
                    Some("journey") => {
                        println!("reading from file {:?}",dir_entry);
                        let ctc=File::open(dir_entry).unwrap();
                        let journey=corr_journeys_builder::parser::read_journey_from_file(ctc);
                        journeys.push(journey);
                    },
                    _=>{}
                }
            }

        }
    }
    let js = JourneyStore {
        journeys
    };
    js.start_with(format!("hello"),Environment::new_rc(user));
}