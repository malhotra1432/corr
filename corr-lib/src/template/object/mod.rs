use crate::template::{Expression, Fillable};
use crate::core::Value;
use crate::core::runtime::Context;
use nom::lib::std::collections::HashMap;
use async_trait::async_trait;
pub mod parser;
#[derive(Clone,Debug,PartialEq)]
pub enum FillableObject{
    WithExpression(Expression),
    WithMap(FillableMapObject),
    WithArray(Vec<FillableObject>)
}
#[derive(Clone,Debug,PartialEq)]
pub enum FillableMapObject{
    WithPairs(Vec<FillablePair>)
}
#[derive(Clone,Debug,PartialEq)]
pub enum FillablePair{
    WithKeyAndValue(String,FillableObject)
}
#[derive(Clone,Debug,PartialEq)]
pub struct FilledPair{
    pub key:String,
    pub value:Value,
}
#[async_trait]
impl Fillable<FilledPair> for FillablePair{
    async fn fill(&self, context: &Context) -> FilledPair {
        match self {
            FillablePair::WithKeyAndValue(key,value)=> FilledPair {
                key:key.clone(),
                value:value.fill(&context).await
            }
        }
    }
}
#[async_trait]
impl Fillable<Value> for Expression {
    async fn fill(&self, context: &Context) -> Value {
        self.evaluate(context).await
    }
}
#[async_trait]
impl Fillable<Value> for FillableMapObject {
    async fn fill(&self, context: &Context) -> Value {
        match self {
            FillableMapObject::WithPairs(pairs)=>{
                let mut value_map = HashMap::new();
                for pair in pairs {
                    let filled_pair=pair.fill(context).await;
                    value_map.insert(filled_pair.key.clone(),filled_pair.value.clone());
                }
                Value::Map(value_map)
            },
        }
    }
}
#[async_trait]
impl Fillable<Value> for Vec<FillableObject>{
    async fn fill(&self, context: &Context) -> Value {
        let mut arr = vec![];
        for value in self {
            arr.push(value.fill(context).await)
        }
        Value::Array(arr)
    }
}
#[async_trait]
impl Fillable<Value> for FillableObject{
    async fn fill(&self, context: &Context) -> Value {
        match self {
            FillableObject::WithExpression(expr)=>expr.fill(context).await,
            FillableObject::WithMap(map)=>map.fill(context).await,
            FillableObject::WithArray(arr)=>arr.fill(context).await
        }
    }
}

#[cfg(test)]
mod tests{
    use crate::core::{DataType, Value};
    use crate::core::proto::{Input, ContinueInput, Output};
    use std::sync::{Arc, Mutex};
    use crate::core::runtime::Context;
    use crate::parser::Parsable;
    use crate::template::object::{FillableObject, FillableMapObject, FillablePair, FilledPair};
    use crate::template::Fillable;
    use nom::lib::std::collections::HashMap;

    #[tokio::test]
    async fn should_fill_fillableobject_when_expression(){
        let txt = r#"object name"#;
        let (_,fo) = FillableObject::parser(txt).unwrap();
        let input=vec![Input::Continue(ContinueInput{name:"name".to_string(),value:"Atmaram".to_string(),data_type:DataType::String})];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let filled = fo.fill(&context).await;
        assert_eq!(filled,Value::String(format!("Atmaram")))
    }

    #[tokio::test]
    async fn should_fill_fillableobject_when_map(){
        let txt = r#"object {"name": name }"#;
        let (_,fo) = FillableObject::parser(txt).unwrap();
        let input=vec![Input::Continue(ContinueInput{name:"name".to_string(),value:"Atmaram".to_string(),data_type:DataType::String})];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let filled = fo.fill(&context).await;
        let mut hm = HashMap::new();
        hm.insert(format!("name"),Value::String(format!("Atmaram")));
        assert_eq!(filled,Value::Map(hm));
    }

    #[tokio::test]
    async fn should_fill_fillableobject_when_array(){
        let txt = r#"object [name,place]"#;
        let (_,fo) = FillableObject::parser(txt).unwrap();
        let input=vec![
            Input::Continue(ContinueInput{name:"name".to_string(),value:"Atmaram".to_string(),data_type:DataType::String}),
            Input::Continue(ContinueInput{name:"place".to_string(),value:"Mumbai".to_string(),data_type:DataType::String})
        ];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let filled = fo.fill(&context).await;
        let mut v = vec![];
        v.push(Value::String(format!("Atmaram")));
        v.push(Value::String(format!("Mumbai")));

        assert_eq!(filled,Value::Array(v));
    }

    #[tokio::test]
    async fn should_fill_fillableobjectmap(){
        let txt = r#"{"name":name, "place":place}"#;
        let (_,fmo) = FillableMapObject::parser(txt).unwrap();
        let input=vec![
            Input::Continue(ContinueInput{name:"name".to_string(),value:"Atmaram".to_string(),data_type:DataType::String}),
            Input::Continue(ContinueInput{name:"place".to_string(),value:"Mumbai".to_string(),data_type:DataType::String})
        ];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let mut hm = HashMap::new();
        hm.insert(format!("name"),Value::String(format!("Atmaram")));
        hm.insert(format!("place"),Value::String(format!("Mumbai")));
        let filled = fmo.fill(&context).await;
        assert_eq!(filled,Value::Map(hm));
    }

    #[tokio::test]
    async fn should_fill_fillablepair(){
        let txt = r#""name":name"#;
        let (_,fp) = FillablePair::parser(txt).unwrap();
        let input=vec![
            Input::Continue(ContinueInput{name:"name".to_string(),value:"Atmaram".to_string(),data_type:DataType::String}),
        ];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let filled = fp.fill(&context).await;
        assert_eq!(filled,FilledPair{
            key:format!("name"),
            value:Value::String(format!("Atmaram"))
        });
    }


}