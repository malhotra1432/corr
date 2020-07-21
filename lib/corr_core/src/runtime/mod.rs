#![feature(async_await)]
use async_trait::async_trait;
use crate::io::StringIO;
use serde::{Serialize, Deserialize, Serializer};
use std::collections::HashMap;
use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::result::Result;
use crate::break_on;
use std::any::Any;
use std::error::Error;
use std::fmt;
use std::sync::{Arc};
use tokio::sync::{RwLock};
use futures::AsyncWriteExt;
use futures::lock::Mutex;
use std::marker::Send;
use futures::future::BoxFuture;
use async_recursion::async_recursion;
pub struct Messanger<T> where T:StringIO{
    pub string_io:Box<T>
}
impl<T> Messanger<T> where T:StringIO{
    pub fn new(str_io:T)->Messanger<T>{
        Messanger{
            string_io:Box::new(str_io)
        }
    }
    pub fn ask(&mut self, var_desc: VariableDesciption)->RawVariableValue {
        self.string_io.write(format!("Please enter value for {} of type {:?}",var_desc.name,var_desc.data_type));
        let line = self.string_io.read();
        RawVariableValue {
            value:Option::Some(line),
            name:var_desc.name.clone(),
            data_type:var_desc.data_type.clone()
        }

    }

    pub fn tell(&mut self, words: String) {
        self.string_io.write(words);
    }
}

#[derive(Debug,PartialEq,Clone,Serialize,Deserialize)]
pub struct VariableDesciption {
    pub name:String,
    pub data_type:VarType
}

#[derive(Debug,PartialEq,Clone,Serialize,Deserialize)]
pub enum VarType{
    String,
    Long,
    Boolean,
    Double,
    Object,
    List,
    Reference
}
#[derive(Debug,Clone)]
pub enum Value{
    String(String),
    Long(i64),
    Boolean(bool),
    Double(f64),
    Object(HashMap<String,Value>),
    Array(Vec<Value>),
    Null,
    Reference(Arc<dyn Any>)
}
impl PartialEq for Value{
    fn eq(&self, other: &Self) -> bool {
        match self {
            Value::Null=>match other {
                Value::Null=>true,
                _=>false
            },
            Value::Double(left)=>match other {
                Value::Double(right)=>if left==right {
                    true
                } else {
                    false
                },
                _=>false
            },
            Value::String(left)=>match other {
                Value::String(right)=>if left==right {
                    true
                } else {
                    false
                },
                _=>false
            },
            Value::Long(left)=>match other {
                Value::Long(right)=>if left==right {
                    true
                } else {
                    false
                },
                _=>false
            },
            Value::Object(left)=>match other {
                Value::Object(right)=>if left==right {
                    true
                } else {
                    false
                },
                _=>false
            },
            Value::Array(left)=>match other {
                Value::Array(right)=>if left==right {
                    true
                } else {
                    false
                },
                _=>false
            },
            Value::Boolean(left)=>match other {
                Value::Boolean(right)=>if left==right {
                    true
                } else {
                    false
                },
                _=>false
            },
            Value::Reference(left)=>match other {
                Value::Reference(right)=>true,
                _=>false
            }
        }
    }
}
impl Value {
    pub fn to_string(&self)->String{
        match self {
            Value::String(val)=>val.clone(),
            Value::Long(val)=>format!("{}",val),
            Value::Double(val)=>format!("{}",val),
            Value::Boolean(val)=>format!("{}",val),
            Value::Null=>format!("null"),
            Value::Array(values)=>{
                let mut str=format!("");
                for value in values{
                    str.push_str(value.to_string().as_str())
                }
                str
            },
            Value::Object(a)=>{
                for key in a.keys() {
                    println!("{}:{}",key,a.get(key).unwrap().to_string())
                }
                "Unkown".to_string()
            },
            Value::Reference(val)=>{
                "Unkown".to_string()
            }
        }
    }
    pub fn from(value:&serde_json::Value)->Value{
        match value {
            serde_json::Value::Null=>Value::Null,
            serde_json::Value::Bool(b)=>Value::Boolean(b.clone()),
            serde_json::Value::String(val)=>Value::String(val.clone()),
            serde_json::Value::Number(n)=>{
                if n.is_i64() {
                    Value::Long(n.as_i64().unwrap())
                } else if n.is_f64() {
                    Value::Double(n.as_f64().unwrap())
                } else {
                    unimplemented!();
                }
            },
            serde_json::Value::Object(obj)=>{
                let mut map=HashMap::new();
                for (key,value) in obj{
                    map.insert(key.clone(),Value::from(value));
                }
                Value::Object(map)
            },
            serde_json::Value::Array(vec)=>{
                let mut new_vec=Vec::new();
                for value in vec{
                    new_vec.push(Value::from(value))
                }
                Value::Array(new_vec)
            }
        }
    }
    pub fn get_associated_var_type(&self)->Option<VarType>{
        match self {
            Value::String(_)=>{
                Option::Some(VarType::String)
            },
            Value::Boolean(_)=>{
                Option::Some(VarType::Boolean)
            },
            Value::Object(_)=>{
                Option::Some(VarType::Object)
            },
            Value::Long(_)=>{
                Option::Some(VarType::Long)
            },
            Value::Array(_)=>{
                Option::Some(VarType::List)
            },
            Value::Double(_)=>{
                Option::Some(VarType::Double)
            },
            Value::Null=>{
                Option::None
            },
            Value::Reference(_)=>Option::Some(VarType::Reference)
        }
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        match self {
            Value::String(val)=>{
                serializer.serialize_str(val.as_str())
            },
            Value::Long(val)=>{
                serializer.serialize_i64(*val)
            },
            Value::Double(val)=>{
                serializer.serialize_f64(*val)
            },
            Value::Null=>{
                serializer.serialize_none()
            },
            Value::Boolean(val)=>{
                serializer.serialize_bool(*val)
            },
            Value::Object(val)=>{
                serializer.serialize_some(val)
            },
            Value::Array(val)=>{
                serializer.serialize_some(val)
            },
            Value::Reference(_)=>{
                serializer.serialize_none()
            }
        }
    }
}
#[derive(Debug,PartialEq,Clone,Serialize,Deserialize)]
pub struct RawVariableValue{
    pub value:Option<String>,
    pub name:String,
    pub data_type:VarType

}
impl RawVariableValue{
    pub fn is_valid(&self)-> bool {
        match &self.value {
            Option::None=> return true,
            _=>{}
        }
        match &self.data_type {
            VarType::String=>true,
            VarType::Long=>match self.value.clone().unwrap().parse::<i64>() {
                Ok(_)=>true,
                _=>false
            },
            VarType::Boolean=>match self.value.clone().unwrap().parse::<bool>() {
                Ok(_)=>true,
                _=>false
            },
            VarType::Double=>match self.value.clone().unwrap().parse::<f64>() {
                Ok(_)=>true,
                _=>false
            }
            _=>false
        }
    }
    pub fn to_value(&self)->Value{
        match &self.value {
            Option::None=> return Value::Null,
            _=>{}
        }
        match &self.data_type {
            VarType::String=>Value::String(self.value.clone().unwrap()),
            VarType::Long=>match self.value.clone().unwrap().parse::<i64>() {
                Ok(val)=>Value::Long(val),
                _=>Value::Null
            },
            VarType::Boolean=>match self.value.clone().unwrap().parse::<bool>() {
                Ok(val)=>Value::Boolean(val),
                _=>Value::Null
            },
            VarType::Double=>match self.value.clone().unwrap().parse::<f64>() {
                Ok(val)=>Value::Double(val),
                _=>Value::Null
            }
            _=>Value::Null
        }
    }
}
#[async_trait]
pub trait ValueProvider{
    async fn save(&self,var:Variable,value:Value);
    async fn read(&mut self,variable:Variable)->Value;

    //    fn iterate<F>(&mut self,refering_as:Variable,to_list:Variable,inner:F) where F:Fn();
    async fn write(&mut self,text:String);
    async fn set_index_ref(&mut self,index_ref_var:Variable,list_ref_var:Variable);
    async fn done(&mut self, str:String);
    async fn load_ith_as(&mut self,i:usize,index_ref_var:Variable,list_ref_var:Variable);
    async fn load_value_as(&mut self,ref_var: Variable, val:Value);
    async fn close(&mut self);
}
pub struct Environment {
    pub channel:Arc<Mutex<RCValueProvider>>
}
impl Environment {
    pub fn new_rc<T>(provider:T)->Environment where T:ValueProvider+Send {
        Environment{
            channel:Arc::new(Mutex::new(RCValueProvider{
                indexes:HashMap::new(),
                reference_store:Arc::new(Mutex::new(HashMap::new())),
                value_store:vec![],
                fallback_provider:Arc::new(Mutex::new(provider))
            }))
        }
    }
}
impl Environment {
    pub async fn error(&self, text: String) {
        self.channel.lock().await.write(text);
        self.channel.lock().await.close();
    }
    pub async fn save(&self,var:Variable,val:Value){
        self.channel.lock().await.save(var,val);
    }
    pub async fn iterate<F>(&self, refering_as: Variable, to_list: Variable, inner: F) where F: Fn(usize) {
        let length = self.channel.lock().await.read(Variable{
            name:format!("{}.size",to_list.name),
            data_type:Option::Some(VarType::Long)
        }).await;
        self.channel.lock().await.set_index_ref(refering_as.clone() ,to_list.clone());
        match length {
            Value::Long(l)=>{
                let size = l as usize;

                for i in 0..size {
                    self.channel.lock().await.load_ith_as(i,refering_as.clone() ,to_list.clone());
                    inner(i);
                    // self.channel.lock().unwrap().drop(refering_as.name.clone());
                }
            },
            _=>{}
        }

    }
    pub async fn iterate_outside_building_inside<F>(&self, refering_as: Variable, to_list: Variable,size:usize, inner: F) where F: Fn(usize) {

        self.channel.lock().await.set_index_ref(refering_as.clone() ,to_list.clone());
        self.channel.lock().await.create_object_at_path(to_list.name.clone(),Arc::new(Mutex::new(Object::new_list_object())));
        for i in 0..size {
            self.channel.lock().await.load_ith_as(i,refering_as.clone() ,to_list.clone());
            inner(i);
            self.channel.lock().await.done(refering_as.name.clone());
        }
    }
    pub async fn build_iterate_outside_building_inside<F,G>(&self, refering_as: Variable, to_list: Variable,size:usize, inner: F)->Vec<G> where F: Fn(usize)->G{
        let mut res=Vec::new();
        self.channel.lock().await.set_index_ref(refering_as.clone() ,to_list.clone());
        self.channel.lock().await.create_object_at_path(to_list.name.clone(),Arc::new(Mutex::new(Object::new_list_object())));
        for i in 0..size {
            self.channel.lock().await.load_ith_as(i,refering_as.clone() ,to_list.clone());
            res.push(inner(i));
            self.channel.lock().await.done(refering_as.name.clone());
        }
        return res;
    }
    pub async fn build_iterate<F,G>(&self, refering_as: Variable, to_list: Variable,mut push_to:Vec<G>, inner: F)->Vec<G> where F: Fn(usize)->G{
        let length = self.channel.lock().await.read(Variable{
            name:format!("{}.size",to_list.name),
            data_type:Option::Some(VarType::Long)
        }).await;
        self.channel.lock().await.set_index_ref(refering_as.clone() ,to_list.clone()).await;
        match length {
            Value::Long(l)=>{
                let size = l as usize;
                for i in 0..size {
                    self.channel.lock().await.load_ith_as(i,refering_as.clone() ,to_list.clone()).await;
                    push_to.push(inner(i));
                    self.channel.lock().await.done(refering_as.name.clone()).await;
                }
                return push_to;
            },
            _=>{return push_to;}
        }

    }
}
#[async_trait]
impl ValueProvider for Environment{
    async fn read(&mut self, variable: Variable) -> Value {
        self.channel.lock().await.read(variable).await
    }

    async fn write(&mut self, text: String) {
        self.channel.lock().await.write(text).await;
    }

    async fn close(&mut self) {
        self.channel.lock().await.close().await
    }
    async fn done(&mut self, str_val: String) {
        self.channel.lock().await.done(str_val).await;
    }
    async fn set_index_ref(&mut self, as_var: Variable, in_var: Variable) {
        self.channel.lock().await.set_index_ref(as_var,in_var).await;
    }

    async fn load_ith_as(&mut self, i: usize, index_ref_var: Variable, list_ref_var: Variable) {
        self.channel.lock().await.load_ith_as(i,index_ref_var,list_ref_var).await;
    }

    async fn save(&self, var: Variable, value: Value) {
        self.channel.lock().await.save(var,value).await;
    }

    async fn load_value_as(&mut self, ref_var: Variable, val: Value) {
        self.channel.lock().await.load_value_as(ref_var,val).await;
    }
}

#[derive(Clone,PartialEq,Debug)]
pub struct Variable{
    pub name:String,
    pub data_type:Option<VarType>
}

#[derive(Debug,Clone)]
pub enum Object{
    Final(Value),
    Object(Arc<Mutex<HashMap<String,Arc<Mutex<Object>>>>>),
    List(Arc<Mutex<Vec<Arc<Mutex<Object>>>>>)
}
impl Object {
    pub fn new_list_object()->Object{
        return Object::List(Arc::new(Mutex::new(vec![])));
    }
    pub fn new_object_object(map:HashMap<String,Arc<Mutex<Object>>>)->Object{
        return Object::Object(Arc::new(Mutex::new(map)))
    }
    pub fn to_value(&self)->Value{
        match self{
            Object::Final(val)=> {
                return val.clone()
            },
            Object::List(inner)=>{
                let mut vec=Vec::new();
                for val in &*(**inner).borrow(){
                    vec.push(val.to_value())
                }
                Value::Array(vec)
            },
            Object::Object(inner)=>{
            let mut map=HashMap::new();
            for (key,val) in &*(**inner).borrow(){
                map.insert(key.clone(),val.to_value());
            }
            Value::Object(map)
            }
        }
    }
}
pub struct RCValueProvider{
    pub fallback_provider:Arc<Mutex<dyn ValueProvider>>,
    pub value_store:Vec<Arc<Mutex<Object>>>,
    pub reference_store:Arc<Mutex<HashMap<String,Arc<Mutex<Object>>>>>,
    pub indexes:HashMap<String,String>,
}
#[derive(Debug)]
pub struct JourneyError{
    message:String
}
impl Error for JourneyError{
}
impl Display for JourneyError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl RCValueProvider {
    #[async_recursion]
    pub async fn get_object_at_path(&self,var:String)->Option<Arc<Mutex<Object>>>{
            if var.contains('.'){
                let (left,right)=break_on(var.clone(),'.').unwrap();
                let opt_rc=self.get_object_at_path(left).await;
                match  opt_rc {
                    Option::Some(rc)=>{
                        let obj=&*rc.lock().await;
                        match obj {
                            Object::Object(obj)=>{
                                let ref_hm=obj.lock().await;
                                let ref_rc=ref_hm.get(&right);
                                match ref_rc {
                                    Option::Some(a_rc)=> Option::Some(a_rc.clone()),
                                    Option::None=>Option::None
                                }
                            },
                            Object::List(lst)=>{
                                if right==format!("size"){
                                    Option::Some(Arc::new(Mutex::new(Object::Final(Value::Long(lst.lock().await.len() as i64)))))
                                } else {
                                    Option::None
                                }
                            },
                            _=>{Option::None}
                        }
                    },
                    Option::None=>{
                        Option::None
                    }
                }

            } else {
                let temp=self.reference_store.lock().await;
                let ref_rc=temp.get(&var);
                match ref_rc {
                    Option::Some(a_rc)=> Option::Some(a_rc.clone()),
                    Option::None=>Option::None
                }

            }


    }
    #[async_recursion]
    pub async fn create_object_at_path(&self,var:String,object:Arc<Mutex<Object>>)->Result<(),JourneyError>{
        if var.contains('.'){
            let (left,right)=break_on(var.clone(),'.').unwrap();
            let rc=self.get_object_at_path(left.clone()).await;
            match rc {
                Option::Some(rc_object)=>{
                    match &*rc_object.lock().await {
                        Object::Object(obj)=>{
                            obj.lock().await.insert(right,object);
                            Ok(())
                        },
                        _=>{
                            Err(JourneyError{
                                message:format!("{} not an object",left.clone())
                            })

                        }
                    }
                },
                Option::None=>{
                    let mut map=HashMap::new();
                    map.insert(right,object);
                    let rc_obj=Arc::new(Mutex::new(Object::new_object_object(map)));
                    self.create_object_at_path(left,rc_obj).await
                }
            }

        } else {
            self.reference_store.lock().await.insert(var.clone(),Arc::clone(&object));
            if self.indexes.contains_key(&var.clone()){
                let rc=self.get_object_at_path(self.indexes.get(&var.clone()).unwrap().clone()).await.unwrap();
                let obj = rc.lock().await;
                match &*obj {
                    Object::List(lst)=>{
                        lst.lock().await.push(Arc::clone(&object));
                        Ok(())
                    },
                    _=>{
                        Err(JourneyError{
                            message:format!("{} not an object",var.clone())
                        })
                    }
                }
            } else {
                Err(JourneyError{
                    message:format!("{} not found",var.clone())
                })
            }
        }

    }

}
#[async_trait]
impl ValueProvider for RCValueProvider{
    
    async fn read(&mut self, var: Variable) -> Value {
        let obj = self.get_object_at_path(var.name.clone()).await;
        match obj {
            Option::Some(rc_value)=>{
                rc_value.lock().await.to_value()
            },
            Option::None =>{
                let opt = break_on(var.name.clone(),'.');
                match opt {
                    Option::Some((left,right))=>{
                        if right.clone() == "size"{
                            self.create_object_at_path(left,Arc::new(Mutex::new(Object::new_list_object()))).await;
                            let val=self.fallback_provider.lock().await.read(var.clone()).await;
                            val
                        } else {
                            let val=self.fallback_provider.lock().await.read(var.clone()).await;
                            self.create_object_at_path(var.name.clone(),Arc::new(Mutex::new(Object::Final(val.clone())))).await;
                            val
                        }
                    },
                    Option::None=>{
                        let val=self.fallback_provider.lock().await.read(var.clone()).await;
                        self.create_object_at_path(var.name.clone(),Arc::new(Mutex::new(Object::Final(val.clone())))).await;
                        val
                    }
                }

            }
        }

    }
    
    async fn write(&mut self, str: String) {
         self.fallback_provider.lock().await.write(str).await
    }
    
    async fn close(&mut self) {
        self.fallback_provider.lock().await.close();
    }
    async fn set_index_ref(&mut self, as_ref:Variable, in_ref:Variable) {
        self.indexes.insert(as_ref.name.clone(), in_ref.name.clone());
    }
    async fn done(&mut self, key: String) {
        let mut keys_to_remove=Vec::new();
        for ref_key in self.reference_store.lock().await.keys()  {
            if ref_key.starts_with(format!("{}.",key).as_str()){
                keys_to_remove.push(ref_key.clone())
            }
        }
        for rm_key in keys_to_remove{
            let mut temp=self.reference_store.lock().await;
            temp.remove(&rm_key);
        }
        let mut temp=self.reference_store.lock().await;
        temp.remove(&key);
    }

    async fn load_ith_as(&mut self, i: usize, index_ref_var: Variable, list_ref_var: Variable) {
        if let Some(val)=self.get_object_at_path(list_ref_var.name.clone()).await{
            let obj = val.lock().await;
            match &*obj {
                Object::List(lst)=>{
                    let mut temp=&mut *self.reference_store.lock().await;
                    if lst.lock().await.len()>i{
                        temp.insert(index_ref_var.name.clone(),lst.lock().await.get(i).unwrap().clone());
                    } else if index_ref_var.data_type.clone().unwrap() == VarType::Object{
                        let map=HashMap::new();
                        let obj = Arc::new(Mutex::new(Object::new_object_object(map)));
                        lst.lock().await.push(obj.clone());
                        temp.insert(index_ref_var.name.clone(),obj);
                    } else if index_ref_var.data_type.clone().unwrap() == VarType::List {
                        let objct = Arc::new(Mutex::new(Object::new_list_object()));
                        lst.lock().await.push(objct.clone());
                        temp.insert(index_ref_var.name.clone(),objct);
                    }
                },
                _=>{unimplemented!()}
            }
        } else {
            let lst=Arc::new(Mutex::new(Object::new_list_object()));
            self.create_object_at_path(list_ref_var.name.clone(),lst.clone());
            let obj = lst.lock().await;
            match  &*obj {
                Object::List(lst)=>{
                    let mut temp=self.reference_store.lock().await;
                    if lst.lock().await.len()>i{
                        temp.insert(index_ref_var.name.clone(),lst.lock().await.get(i).unwrap().clone());
                    } else if index_ref_var.data_type.clone().unwrap() == VarType::Object{
                        let map=HashMap::new();
                        let obj = Arc::new(Mutex::new(Object::new_object_object(map)));
                        lst.lock().await.push(obj.clone());
                        temp.insert(index_ref_var.name.clone(),obj);
                    } else if index_ref_var.data_type.clone().unwrap() == VarType::List {
                        let objct = Arc::new(Mutex::new(Object::new_list_object()));
                        lst.lock().await.push(objct.clone());
                        temp.insert(index_ref_var.name.clone(),objct);
                    }
                },
                _=>{unimplemented!()}
            }
        }
    }
    async fn load_value_as(&mut self,ref_var: Variable, val:Value) {
        let mut temp=self.reference_store.lock().await;
        temp.insert(ref_var.name.clone(),Arc::new(Mutex::new(Object::Final(val))));

    }
    async fn save(&self, var: Variable, value: Value) {
        self.create_object_at_path(var.name.clone(),Arc::new(Mutex::new(Object::Final(value)))).await;
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use crate::runtime::{RCValueProvider, Object};
    use crate::runtime::Variable;
    use crate::runtime::Environment;
    use crate::runtime::{ValueProvider, VarType};
    use std::collections::HashMap;
    use crate::runtime::Value;
    use std::cell::RefCell;
    use std::sync::{Arc, Mutex};

    extern crate serde_json;

    #[test]
    fn should_serialize_string_value() {
        assert_eq!(serde_json::to_string(&Value::String(format!("hello"))).unwrap(), format!("\"hello\""))
    }

    #[test]
    fn should_serialize_long_value() {
        assert_eq!(serde_json::to_string(&Value::Long(34)).unwrap(), format!("34"))
    }

    #[test]
    fn should_serialize_double_value() {
        assert_eq!(serde_json::to_string(&Value::Double(34.00)).unwrap(), format!("34.0"))
    }

    #[test]
    fn should_serialize_null_value() {
        assert_eq!(serde_json::to_string(&Value::Null).unwrap(), format!("null"))
    }

    #[test]
    fn should_serialize_object_value() {
        let mut map = HashMap::new();
        map.insert(format!("hello"), Value::String(format!("hello")));
        assert_eq!(serde_json::to_string(&Value::Object(map)).unwrap(), format!(r#"{{"hello":"hello"}}"#))
    }

    #[test]
    fn should_serialize_array_value() {
        let mut array = Vec::new();
        array.push(Value::String(format!("hello")));
        assert_eq!(serde_json::to_string(&Value::Array(array)).unwrap(), format!(r#"["hello"]"#))
    }
    #[async_trait]

    impl ValueProvider for MockProvider {

        async fn read(&mut self, _: Variable) -> Value {
            let ret = self.1[self.0].clone();
            self.0 += 1;
            ret
        }
        async fn write(&mut self, str: String) { println!("{}", str) }
        async fn close(&mut self) {}
        async fn set_index_ref(&mut self, _: Variable, _: Variable) {}
        async fn done(&mut self, _: String) {}

        async fn load_ith_as(&mut self, _i: usize, _index_ref_var: Variable, _list_ref_var: Variable) {}

        async fn save(&self, _var: Variable, _value: Value) {
            unimplemented!()
        }

        async fn load_value_as(&mut self, _ref_var: Variable, _val: Value) {
            unimplemented!()
        }
    }

    #[derive(Debug)]
    struct MockProvider(usize, Vec<Value>);

    #[test]
    fn should_iterate_over_runtime() {
        let rt = Environment::new_rc(MockProvider(0, vec![Value::Long(3),
                                                          Value::String(format!("Atmaram 0")),
                                                          Value::Long(2),
                                                          Value::String(format!("Atmaram 00")),
                                                          Value::String(format!("Atmaram 01")),
                                                          Value::String(format!("Atmaram 1")),
                                                          Value::Long(2),
                                                          Value::String(format!("Atmaram 10")),
                                                          Value::String(format!("Atmaram 11")),
                                                          Value::String(format!("Atmaram 2")),
                                                          Value::Long(2),
                                                          Value::String(format!("Atmaram 20")),
                                                          Value::String(format!("Atmaram 21"))
        ]));
        let mch = Arc::clone(&rt.channel);
        rt.iterate(Variable {
            name: format!("hobby"),
            data_type: Option::Some(VarType::Object)
        }, Variable {
            name: format!("hobbies"),
            data_type: Option::Some(VarType::List)
        }, |i| {
            assert_eq!(mch.borrow_mut().read(Variable {
                name: format!("hobby.name"),
                data_type: Option::Some(VarType::String)
            }), Value::String(format!("Atmaram {}", i)));
            rt.iterate(Variable {
                name: format!("category"),
                data_type: Option::Some(VarType::Object)
            }, Variable {
                name: format!("hobby.categories"),
                data_type: Option::Some(VarType::List)
            }, |j| {
                assert_eq!(mch.borrow_mut().read(Variable {
                    name: format!("category.name"),
                    data_type: Option::Some(VarType::String)
                }), Value::String(format!("Atmaram {}{}", i, j)))
            });
        })
    }

    #[test]
    fn should_read_same_variable_from_rc_provider_multiple_times() {
        let mut rcp = RCValueProvider {
            value_store: Vec::new(),
            reference_store: Arc::new(Mutex::new(HashMap::new())),
            fallback_provider: Box::new(MockProvider(0, vec![
                Value::String(format!("Atmaram"))
            ])),
            indexes: HashMap::new()
        };
        let var = Variable {
            name: format!("name"),
            data_type: Option::Some(VarType::String)
        };
        assert_eq!(rcp.read(var.clone()), Value::String(format!("Atmaram")));
        assert_eq!(rcp.read(var.clone()), Value::String(format!("Atmaram")));
    }

    #[test]
    fn should_size_from_rc_provider() {
        let mut rcp = RCValueProvider {
            value_store: Vec::new(),
            reference_store: Arc::new(Mutex::new(HashMap::new())),
            fallback_provider: Box::new(MockProvider(0, vec![
                Value::Long(2),
                Value::String(format!("Atmaram"))
            ])),
            indexes: HashMap::new()
        };
        let var = Variable {
            name: format!("name.size"),
            data_type: Option::Some(VarType::String)
        };
        assert_eq!(rcp.read(var.clone()), Value::Long(2));
    }

    #[test]
    fn should_iterate_over_runtime_reading_values() {
        let rt = Environment {
            channel: Arc::new(Mutex::new((
                RCValueProvider {
                    value_store: Vec::new(),
                    reference_store: Arc::new(Mutex::new(HashMap::<String,Arc<Mutex<Object>>>::new())),
                    fallback_provider: Box::new(MockProvider(0, vec![
                        Value::Long(2),
                        Value::String(format!("Atmaram 0")),
                        Value::String(format!("Atmaram 1"))
                    ])),
                    indexes: HashMap::new()
                }
            )))
        };
        let mch = Arc::clone(&rt.channel);
        rt.iterate(Variable {
            name: format!("hobby"),
            data_type: Option::Some(VarType::String)
        }, Variable {
            name: format!("hobbies"),
            data_type: Option::None
        }, |i| {
            assert_eq!(mch.borrow_mut().read(Variable {
                name: format!("hobby.name"),
                data_type: Option::Some(VarType::String)
            }), Value::String(format!("Atmaram {}", i)));
        });
        rt.iterate(Variable {
            name: format!("hobby"),
            data_type: Option::Some(VarType::String)
        }, Variable {
            name: format!("hobbies"),
            data_type: Option::None
        }, |i| {
            println!("helo");
            assert_eq!(mch.borrow_mut().read(Variable {
                name: format!("hobby.name"),
                data_type: Option::Some(VarType::String)
            }), Value::String(format!("Atmaram {}", i)));
        });
    }
    #[test]
    fn should_convert_long() {
        let val=serde_json::Value::from(12);
        assert_eq!(Value::from(&val),Value::Long(12))
    }
    #[test]
    fn should_convert_null() {
        let val=serde_json::Value::Null;
        assert_eq!(Value::from(&val),Value::Null)
    }
}
