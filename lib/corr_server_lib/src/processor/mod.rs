extern crate tokio;
extern crate websocket;
extern crate corr_core;
extern crate serde_json;
extern crate corr_templates;
extern crate corr_journeys;
use std::thread;
use websocket::sync::Server;
use websocket::OwnedMessage;
use websocket::sync::Client;
use self::websocket::websocket_base::stream::sync::Splittable;
use std::convert::TryInto;
use self::corr_core::{Action, DesiredAction, VariableDesciption, VarType, RawVariableValue, Channel, Value, Variable, Runtime};
use corr_journeys::Journey;
use std::thread::Thread;
use std::time::Duration;
use std::process::exit;
use self::corr_templates::json::Fillable;
use std::borrow::BorrowMut;
use self::corr_journeys::{Executable, JourneyStore, Interactable};
use std::cell::RefCell;
use std::rc::Rc;

pub struct SocketClient<T>(T) where T:IO;

pub trait IO {
    fn send(&mut self,desired_action:DesiredAction);
    fn wait_for_action(&mut self)->Action;
    fn close(&mut self);
}
impl<T> IO for Client<T> where T:std::io::Read+std::io::Write+Splittable{
    fn send(&mut self,desired_action: DesiredAction) {
        self.send_message(&OwnedMessage::Text(serde_json::to_string(&desired_action).unwrap()));
    }

    fn wait_for_action(&mut self)-> Action {
        let message = self.recv_message().unwrap();
        match message {
            OwnedMessage::Close(_) => {
                Action::Quit
            }
            OwnedMessage::Ping(ping) => {
                Action::Ping
            }
            OwnedMessage::Text(val) => {
                let var:RawVariableValue=serde_json::from_str(&val.as_str()).unwrap();
                Action::Told(var)
            },
            OwnedMessage::Pong(pong)=>{
                Action::Pong
            },
            OwnedMessage::Binary(data)=>{
                Action::Ignorable
            }
        }
    }

    fn close(&mut self) {
    }
}
impl<T> Channel for SocketClient<T> where T:IO{

    fn read(&mut self, variable: Variable) -> Value {
        let mut desired_action = DesiredAction::Tell(VariableDesciption{
           name: variable.name.clone(),
            data_type:match variable.data_type {
                Option::None=>{
                    VarType::String
                }
                Option::Some(dt)=>{
                    dt
                }
            }
        });

        self.0.send(desired_action.clone());
        while let client_action=self.0.wait_for_action() {
            match &client_action {
                Action::Told(var)=>{
                    if var.is_valid() {
                        println!("told me variable {:?}", var);
                        return var.to_value()
                    } else {
                        self.0.send(DesiredAction::Listen(format!("Not Valid {:?}", var.data_type)));
                        self.0.send(desired_action.clone());
                        continue;
                    }
                },
                _=>{
                    continue;
                }
            }
        }
        return Value::Null


    }
    fn write(&mut self, text: String) {
        self.0.send(DesiredAction::Listen(text))
    }

    fn close(&mut self) {
        self.0.send(DesiredAction::Quit);
        while let client_action=self.0.wait_for_action() {
            match &client_action {
                Action::Quit=>{
                    self.0.close();
                    return;
                },
                _=>{
                    continue;
                }
            }
        }

    }
}
pub fn start<T>(mut io:SocketClient<T>) where T:IO{
    let journeys=vec![
        Journey{
            name:format!("Tell me your name")
        },
        Journey{
            name:format!("Tell me your age")
        }
    ];
    let js = JourneyStore {
        journeys
    };
    js.start_with(format!("hello"),Runtime{ channel:Rc::new(RefCell::new(io))});
}
pub fn create_server() {
    let server = Server::bind("127.0.0.1:9876").unwrap();

    for request in server.filter_map(Result::ok) {
        // Spawn a new thread for each connection.
        thread::spawn(|| {
            if !request.protocols().contains(&"rust-websocket".to_string()) {
                request.reject().unwrap();
                return;
            }

            let mut client = request.use_protocol("rust-websocket").accept().unwrap();

            let ip = client.peer_addr().unwrap();

            println!("Connection from {}", ip);
            start(SocketClient(client))
        });
    }
}