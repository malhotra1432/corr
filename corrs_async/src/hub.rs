use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::{mpsc, RwLock};
use warp::ws::{Message, WebSocket};
use std::collections::HashMap;
use futures::stream::SplitStream;
use futures::{AsyncReadExt, StreamExt};
use crate::proto::{InputParcel, Input, Output, StartInput, KnowThatOutput, TellMeOutput};
use crate::proto::Result;
use std::ops::Generator;
use futures::executor::block_on;
use corr_core::runtime::{ValueProvider, Variable, Value, VariableDesciption, VarType};
use std::time::Duration;

type Users = Arc<RwLock<HashMap<usize, User>>>;
pub struct Hub{
    pub users:Users
}
pub struct User {
    pub tx:mpsc::UnboundedSender<Result<Message>>,
    pub user_ws_rx:SplitStream<WebSocket>
}
impl User {
    pub fn new(tx:mpsc::UnboundedSender<Result<Message>>,mut user_ws_rx:SplitStream<WebSocket>)->User{
        User{
            tx,
            user_ws_rx
        }
    }
    pub fn send(&self,output:Output){
        if let Err(_disconnected) = self.tx.send(Ok(Message::text(serde_json::to_string(&output).unwrap()))) {

        }
    }
    pub async fn getMessage(&mut self)->Input{
            let mut ret=Input::Start(StartInput{filter:format!("hello")});
            loop{
                if let Some(result) = self.user_ws_rx.next().await {
                    let message = match result {
                        Ok(msg) => msg,
                        Err(e) => {
                            eprintln!("websocket error( {}", e);
                            break;
                        }
                    };
                    let input:Input = serde_json::from_str(message.to_str().unwrap()).unwrap();
                    eprintln!("Got Message{:?}",input);
                    ret=input;
                    break;
                }
            };
        return ret;
    }
}
impl Hub {
    pub fn new() -> Self {
        Hub {
            users:Users::default()
        }
    }
}
impl ValueProvider for User{
    fn save(&self, var: Variable, value: Value) {

    }
    fn async read(&mut self, variable: Variable) -> Value {
        let output = Output::TellMe(TellMeOutput{ variableDescription:VariableDesciption{
            name: variable.name.clone(),
            data_type:match variable.data_type {
                Option::None=>{
                    VarType::String
                }
                Option::Some(dt)=>{
                    dt
                }
            }
        }});
        self.send(output.clone());
        loop{
            let input=self.getMessage().await;
            match &input {
                Input::Continue(ci)=>{
                    if ci.rawVariableValue.is_valid() {
                        return ci.rawVariableValue.to_value()
                    } else {
                        self.send(Output::KnowThat(KnowThatOutput{ phrase:format!("Not Valid {:?}", ci.rawVariableValue.data_type)}));
                        self.send(output.clone());
                        continue;
                    }
                },
                _=>{
                    continue;
                }
            }
        }
    }

    fn write(&mut self, text: String) {
        eprintln!("Sending {}",text);
        self.send(Output::KnowThat(KnowThatOutput{
            phrase:text
        }))
    }

    fn set_index_ref(&mut self, index_ref_var: Variable, list_ref_var: Variable) {

    }

    fn done(&mut self, str: String) {

    }

    fn load_ith_as(&mut self, i: usize, index_ref_var: Variable, list_ref_var: Variable) {

    }

    fn load_value_as(&mut self, ref_var: Variable, val: Value) {

    }

    fn close(&mut self) {

    }
}