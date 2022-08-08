use crate::data_set_loader::DataSetLoader;
use serde_json::Value;

pub struct CSVDataSet {
    pub data_set: String,
}

impl DataSetLoader for CSVDataSet {
    fn new(data_set: &'static str) -> Self {
        Self {
            data_set: data_set.to_string(),
        }
    }

    fn load(&self) -> Vec<Value> {
        let data_set_value = serde_json::from_str(&self.data_set).unwrap_or(Value::Null);
        let mut result = vec![];

        if let Value::Array(rows_value) = data_set_value {
            let header_row_v = rows_value.get(0).unwrap().as_array().unwrap();
            let header_row = header_row_v
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect::<Vec<&str>>();

            for row_value in &rows_value[1..rows_value.len()] {
                if let Value::Array(row_v) = row_value {
                    let row: Vec<Value> = row_v.iter().map(Self::parse_value).collect();
                    result.push(header_row.iter().cloned().zip(row.clone()).collect());
                }
            }
        }
        result
    }
}

impl CSVDataSet {
    fn parse_value(value: &Value) -> Value {
        if value.is_string() {
            if let Ok(parsed) = serde_json::from_str::<Value>(value.as_str().unwrap()) {
                return parsed;
            }
        }

        value.clone()
    }
}
