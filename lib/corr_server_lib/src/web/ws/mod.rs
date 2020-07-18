use std::sync::{Arc};
use std::collections::HashMap;
use warp::{ws::Message, Filter, Rejection, Reply};
use futures::{FutureExt, StreamExt, Future};
use warp::filters::ws::WebSocket;
use tokio::sync::{mpsc,RwLock};
use crate::web::Result;
use corr_core::runtime::{ValueProvider, Variable, Value, VariableDesciption, VarType, Environment, RawVariableValue};
use corr_websocket::{DesiredAction, Action};
use app_dirs2::{AppDataType, app_root, AppInfo};
use std::fs::File;
use corr_journeys::{JourneyStore, Interactable};
use async_trait::async_trait;
use std::thread;
use futures::task::{RawWakerVTable, Waker, Context, Poll};
use futures::future::Ready;
use futures::executor::block_on;
use futures::stream::SplitStream;
use websocket::sender::Sender;
use std::error::Error;

pub type Clients = Arc<RwLock<HashMap<String, Client>>>;
const APP_INFO: AppInfo = AppInfo{name: "corrs", author: "Atmaram Naik"};
#[derive(Debug, Clone)]
pub struct Client {
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

pub async fn handler(ws: warp::ws::Ws, clients: Clients) -> Result<impl Reply> {
    println!("Called handler");
    Ok(ws.on_upgrade(move |socket| client_connection(socket, clients)))
    // let client = clients.read().await.get(&id).cloned();
    // match client {
    //     Some(c) => Ok(ws.on_upgrade(move |socket| client_connection(socket, id, clients, c))),
    //     None => Err(warp::reject::not_found()),
    // }
}
struct WebSocketConnection{
    client_ws_rcv:SplitStream<WebSocket>,
    client_sender:tokio::sync::mpsc::UnboundedSender<std::result::Result<Message,warp::Error>>,
}
impl WebSocketConnection{
    pub fn start(ws:WebSocket)->WebSocketConnection{
        let (client_ws_sender, mut client_ws_rcv) = ws.split();
        let (client_sender, client_rcv) = mpsc::unbounded_channel();
        tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
            if let Err(e) = result {
                eprintln!("error sending websocket msg: {}", e);
            }
        }));
        WebSocketConnection{
            client_ws_rcv,
            client_sender
        }
    }
}
pub async fn client_connection(ws: WebSocket, clients: Clients) {
    let wsc=ConnectionHolder(WebSocketConnection::start(ws));
    start(wsc);
}
async fn client_msg(msg: Message, clients: &Clients) {
    println!("received message from {:?}", msg);
    let message = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };
}

#[async_trait]
pub trait IO {
    async fn send(&mut self,desired_action:DesiredAction);
    async fn wait_for_action(&mut self)->Action;
    async fn close(&mut self);
}
pub struct ConnectionHolder<T>(T) where T: IO;
#[async_trait]
impl IO for WebSocketConnection {
    async fn send(&mut self, desired_action: DesiredAction) {
        println!("sending {:?}",&desired_action);
        //
        self.client_sender.send(Ok(Message::binary(serde_json::to_string(&desired_action).unwrap().into_bytes())));
        println!("sent");
    }
    async fn wait_for_action(&mut self) -> Action {
        let mut res=Option::None;
        loop {
            if let Some(result)=self.client_ws_rcv.next().await{
                res = Option::Some(match result {
                    Ok(message) => {
                        let var: RawVariableValue = serde_json::from_str(message.to_str().unwrap()).unwrap();
                        let action=Action::Told(var);
                        action
                    },
                    Err(_) => {
                        Action::Quit
                    }
                }
                );
                break;
            } else {
                continue;
            }

        } {

        }
        println!("{:?}",res);
        res.unwrap()

    }

    async fn close(&mut self) {

    }
}
    impl<T> ValueProvider for ConnectionHolder<T> where T: IO {

        fn read(&mut self, variable: Variable) -> Value {
            let desired_action = DesiredAction::Tell(VariableDesciption {
                name: variable.name.clone(),
                data_type: match variable.data_type {
                    Option::None => {
                        VarType::String
                    }
                    Option::Some(dt) => {
                        dt
                    }
                }
            });

            block_on(self.0.send(desired_action.clone()));
            loop {
                let client_action = block_on(self.0.wait_for_action());
                match &client_action {
                    Action::Told(var) => {
                        if var.is_valid() {
                            println!("Got valid value");
                            return var.to_value()
                        } else {
                            self.0.send(DesiredAction::Listen(format!("Not Valid {:?}", var.data_type)));
                            self.0.send(desired_action.clone());
                            continue;
                        }
                    },
                    _ => {
                        continue;
                    }
                }
            }
        }
        fn write(&mut self, text: String) {
            println!("Called Write on Connection Holder");
            block_on(self.0.send(DesiredAction::Listen(text)));
        }

        fn close(&mut self) {
            block_on(self.0.send(DesiredAction::Quit));
            loop {
                let client_action = block_on(self.0.wait_for_action());
                match &client_action {
                    Action::Quit => {
                        self.0.close();
                        return;
                    },
                    _ => {
                        continue;
                    }
                }
            }
        }
        fn set_index_ref(&mut self, _: Variable, _: Variable) {}
        fn drop(&mut self, _: String) {}

        fn load_ith_as(&mut self, _i: usize, _index_ref_var: Variable, _list_ref_var: Variable) {}

        fn save(&self, _var: Variable, _value: Value) {}

        fn load_value_as(&mut self, _ref_var: Variable, _val: Value) {}
    }
    pub fn start<T: 'static>(io: ConnectionHolder<T>) where T: IO {
        let mut journeys = Vec::new();
        let mut app_path = app_root(AppDataType::UserConfig, &APP_INFO).unwrap();
        let path = app_path.join("journeys");
        println!("{:?}", path);
        for dir_entry in std::fs::read_dir(path).unwrap() {
            let dir_entry = dir_entry.unwrap().path();
            if dir_entry.is_file() {
                if let Some(extention) = dir_entry.extension() {
                    match extention.to_str() {
                        Some("journey") => {
                            println!("reading from file {:?}", dir_entry);
                            let ctc = File::open(dir_entry).unwrap();
                            let journey = corr_journeys_builder::parser::read_journey_from_file(ctc);
                            journeys.push(journey);
                        },
                        _ => {}
                    }
                }
            }
        }
        let js = JourneyStore {
            journeys
        };
        js.start_with(format!("hello"), Environment::new_rc(io));
    }

