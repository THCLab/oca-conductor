use crate::data_set_loader::DataSetLoader;
use serde_json::Value;

pub struct JSONDataSet {
    pub data_set: String,
}

impl DataSetLoader for JSONDataSet {
    fn new(data_set: &'static str) -> Self {
        Self {
            data_set: data_set.to_string(),
        }
    }

    fn load(&self) -> Vec<Value> {
        match serde_json::from_str(&self.data_set).unwrap_or(Value::Null) {
            Value::Array(data_set_array) => data_set_array,
            Value::Object(data_set_object) => vec![Value::Object(data_set_object)],
            _ => {
                vec![]
            }
        }
    }
}
