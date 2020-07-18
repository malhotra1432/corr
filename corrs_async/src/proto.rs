use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::result;
use warp::Error;
use corr_core::runtime::{VariableDesciption, RawVariableValue};

pub type Result<T> = result::Result<T, Error>;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum Input {
    #[serde(rename = "start")]
    Start(StartInput),
    #[serde(rename = "continue")]
    Continue(ContinueInput),
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Output {
    #[serde(rename = "knowThat")]
    KnowThat(KnowThatOutput),
    #[serde(rename = "tellMe")]
    TellMe(TellMeOutput),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartInput {
    pub filter: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinueInput {
    pub rawVariableValue: RawVariableValue,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowThatOutput {
    pub phrase: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TellMeOutput {
    pub variableDescription:VariableDesciption
}

#[derive(Debug, Clone)]
pub struct InputParcel {
    pub client_id: usize,
    pub input: Input,
}

impl InputParcel {
    pub fn new(client_id: usize, input: Input) -> Self {
        InputParcel { client_id, input }
    }
}

#[derive(Debug, Clone)]
pub struct OutputParcel {
    pub client_id: usize,
    pub output: Output,
}

impl OutputParcel {
    pub fn new(client_id: usize, output: Output) -> Self {
        OutputParcel { client_id, output }
    }
}