pub mod csv_data_set;
pub mod json_data_set;

use serde_json::Value;

pub trait DataSetLoader {
    fn new(data_set: &'static str) -> Self
    where
        Self: Sized;
    fn load(&self) -> Vec<Value>;
}
