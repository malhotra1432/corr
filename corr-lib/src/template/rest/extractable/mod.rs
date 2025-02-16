pub mod parser;
use crate::core::runtime::Context;
use crate::template::VariableReferenceName;
use crate::template::object::extractable::{ExtractableObject, Extractable};
use async_trait::async_trait;
// use warp::hyper::http::HeaderValue;
use crate::core::Value;
use hyper::{header::HeaderValue};
// use isahc::prelude::Response;
// use isahc::Body;

use hyper::HeaderMap;
use crate::template::rest::MultipartField;
use crate::template::form::extractable::ExtractableForm;
// use formdata::FormData;

#[derive(Debug, Clone,PartialEq)]
pub struct ExtractableHeaders {
    pub headers:Vec<ExtractableHeaderPair>
}

#[derive(Debug, Clone,PartialEq)]
pub struct ExtractableHeaderPair {
    pub key:String,
    pub value: ExtractableHeaderValue
}
#[derive(Debug, Clone,PartialEq)]

pub enum ExtractableHeaderValue {
    WithVariableReference(VariableReferenceName)
}
pub struct CorrResponse{
    pub body:String,
    pub headers:HeaderMap,
    pub status:u16
}
pub struct Header{
    pub key:&'static str,
    pub value:&'static str,
}
impl Header{
    pub fn new(key:&'static str,value:&'static str)->Self{
        Header{
            key,
            value
        }
    }
}
impl CorrResponse {
    pub fn new(body:&str,headers:Vec<Header>,status:u16)->Self{
        let mut hm = HeaderMap::new();
        for header in headers {
            hm.insert(header.key,HeaderValue::from_str(header.value).unwrap());
        }

        CorrResponse{
            body:body.to_string(),
            headers:hm,
            status
        }
    }
}
#[derive(Debug, Clone,PartialEq)]
pub enum ExtractableBody {
    WithObject(ExtractableObject),
    WithForm(ExtractableForm)
}
pub enum RestBody {
    JSON(serde_json::Value),
    Form(Vec<MultipartField>)
    // FormData(FormData)
}
#[derive(Debug, Clone,PartialEq)]
pub struct ExtractableRestData {
    pub body:Option<ExtractableBody>,
    pub headers:Option<ExtractableHeaders>
}
#[async_trait]
impl Extractable<RestBody> for ExtractableBody {
    async fn extract_from(&self, context: &Context, value: RestBody) {
        match self {
            ExtractableBody::WithObject(eb)=>{
                match value {
                    RestBody::JSON(body)=>{
                        eb.extract_from(context,body).await
                    },
                    _=>{}
                }

            },
            ExtractableBody::WithForm(form)=>{
                if let RestBody::Form(fields) = value {
                    form.extract_from(context,fields).await
                }
            }
        }
    }
}
#[async_trait]
impl Extractable<CorrResponse> for ExtractableRestData {
    async fn extract_from(&self, context: &Context, value: CorrResponse) {
            if let Some(eb) = &self.body{
                match eb {
                    ExtractableBody::WithObject(_)=>{
                        eb.extract_from(context, RestBody::JSON(serde_json::from_str::<serde_json::Value>(value.body.as_str()).unwrap_or(serde_json::Value::Null))).await;
                    },
                    _=>{}
                }
            }
            if let Some(eh) = &self.headers{
                eh.extract_from(context,value).await
            }
    }
}
#[async_trait]
impl Extractable<(serde_json::Value,HeaderMap)> for ExtractableRestData {
    async fn extract_from(&self, context: &Context, (body,headers): (serde_json::Value,HeaderMap)) {
        if let Some(eb) = &self.body{
            match eb {
                ExtractableBody::WithObject(_)=>{
                    eb.extract_from(context, RestBody::JSON(body)).await;
                }
                _=>{}
            }
        }
        if let Some(eh) = &self.headers{
            eh.extract_from(context,headers).await
        }
    }
}
#[async_trait]
impl Extractable<(Vec<MultipartField>,HeaderMap)> for ExtractableRestData {
    async fn extract_from(&self, context: &Context, (fields,headers): (Vec<MultipartField>,HeaderMap)) {
        if let Some(eb) = &self.body{
            match eb {
                ExtractableBody::WithForm(_)=>{
                    eb.extract_from(context, RestBody::Form(fields)).await;
                },
                _=>{}
            }
        }
        if let Some(eh) = &self.headers{
            eh.extract_from(context,headers).await
        }
    }
}
#[async_trait]
impl Extractable<CorrResponse> for ExtractableHeaders {
    async fn extract_from(&self, context: &Context, value: CorrResponse) {
        for header in &self.headers {
            if let Some(hv) = value.headers.get(header.key.clone()){
                header.value.extract_from(context,hv.clone()).await
            }
        }
    }
}
#[async_trait]
impl Extractable<HeaderMap> for ExtractableHeaders {
    async fn extract_from(&self, context: &Context, value: HeaderMap) {
        for header in &self.headers {
            if let Some(hv) = value.get(&header.key){
                header.value.extract_from(context,hv.clone()).await
            }
        }
    }
}
#[async_trait]
impl Extractable<HeaderValue> for ExtractableHeaderValue {
    async fn extract_from(&self, context: &Context, value: HeaderValue) {
        match self {
            ExtractableHeaderValue::WithVariableReference(var)=>{
                if let Ok(hv)=value.to_str(){
                    context.define(var.to_string(),Value::String(hv.to_string())).await
                }
            }
        }
    }
}

#[cfg(test)]
mod tests{
    use crate::core::{Value};
    use crate::core::proto::{Output};
    use std::sync::{Arc, Mutex};
    use crate::core::runtime::{Context};
    use crate::parser::Parsable;
    use crate::template::rest::extractable::{ExtractableHeaderValue, ExtractableHeaders, CorrResponse, Header, ExtractableRestData, ExtractableBody, RestBody};
    use crate::template::object::extractable::Extractable;
    use warp::hyper::http::HeaderValue;

    #[tokio::test]
    async fn should_extract_extractableresponseheadervalue(){
        let text=r#"token"#;
        let (_,ep) = ExtractableHeaderValue::parser(text).unwrap();
        let input=vec![];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        ep.extract_from(&context,HeaderValue::from_static("XYZABC")).await;
        assert_eq!(context.get_var_from_store(format!("token")).await,Option::Some(Value::String(format!("XYZABC"))))
    }
    #[tokio::test]
    async fn should_extract_extractableresponseheaderheaders(){
        let text=r#"{"Authorization":token,"X-API-KEY": api_key }"#;
        let (_,ep) = ExtractableHeaders::parser(text).unwrap();
        let input=vec![];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let cr= CorrResponse::new("",vec![Header::new("Authorization","ABCDXYZ"),
          Header::new("X-API-KEY","SomethingIsBetterThanNothing")
        ],200);
        ep.extract_from(&context,cr).await;
        assert_eq!(context.get_var_from_store(format!("token")).await,Option::Some(Value::String(format!("ABCDXYZ"))));
        assert_eq!(context.get_var_from_store(format!("api_key")).await,Option::Some(Value::String(format!("SomethingIsBetterThanNothing"))));
    }
    #[tokio::test]
    async fn should_extract_extractableresponse(){
        let text=r#"body object {"name":name } and headers {"Authorization":token,"X-API-KEY": api_key }"#;
        let (_,ep) = ExtractableRestData::parser(text).unwrap();
        let input=vec![];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let cr= CorrResponse::new(r#"{"name":"Atmaram"}"#,vec![Header::new("Authorization","ABCDXYZ"),
                                          Header::new("X-API-KEY","SomethingIsBetterThanNothing")
        ],200);
        ep.extract_from(&context,cr).await;
        assert_eq!(context.get_var_from_store(format!("name")).await,Option::Some(Value::String(format!("Atmaram"))));
        assert_eq!(context.get_var_from_store(format!("token")).await,Option::Some(Value::String(format!("ABCDXYZ"))));
        assert_eq!(context.get_var_from_store(format!("api_key")).await,Option::Some(Value::String(format!("SomethingIsBetterThanNothing"))));
    }
    #[tokio::test]
    async fn should_extract_extractableresponsebody(){
        let text=r#"object {"place":place }"#;
        let (_,ep) = ExtractableBody::parser(text).unwrap();
        let input=vec![];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        ep.extract_from(&context, RestBody::JSON(serde_json::from_str::<serde_json::Value>(r#"{"place":"Pune"}"#).unwrap())).await;
        assert_eq!(context.get_var_from_store(format!("place")).await,Option::Some(Value::String(format!("Pune"))));
    }
}