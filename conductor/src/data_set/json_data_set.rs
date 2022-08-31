use crate::data_set::DataSet;
use crate::transformator::Operation;
use oca_rust::state::oca::OCA;
use serde::{Serialize, Serializer};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct JSONDataSet {
    pub plain: String,
}

impl Serialize for JSONDataSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.plain)
    }
}

impl DataSet for JSONDataSet {
    fn new(plain: String) -> Self {
        Self { plain }
    }

    fn load(&self, _attribute_types: BTreeMap<String, String>) -> Vec<Value> {
        match serde_json::from_str(&self.plain).unwrap_or(Value::Null) {
            Value::Array(data_set_array) => data_set_array,
            Value::Object(data_set_object) => vec![Value::Object(data_set_object)],
            _ => {
                vec![]
            }
        }
    }

    fn transform_schema(&self, _mappings: BTreeMap<String, String>) -> Box<dyn DataSet> {
        Box::new(Self::new(self.plain.clone()))
    }

    fn transform_data(
        &self,
        oca: &OCA,
        entry_code_mappings: BTreeMap<String, BTreeMap<String, String>>,
        unit_transformation_operations: BTreeMap<String, Vec<Operation>>,
    ) -> Box<dyn DataSet> {
        let mut transformed_data_set = vec![];

        for record in self.load(oca.capture_base.attributes.clone()) {
            let record_map = record.as_object().unwrap();
            let mut transformed_data = Map::new();
            for (k, v) in record_map {
                let key = k.to_string();
                let mut value = v.clone();
                if let Some(mapped_entries) = entry_code_mappings.get(k) {
                    match value {
                        Value::Array(ref values_vec) => {
                            let mut mapped_values = vec![];
                            for v in values_vec {
                                match mapped_entries.get(v.as_str().unwrap()) {
                                    Some(mapped_entry) => {
                                        mapped_values.push(Value::String(mapped_entry.to_string()));
                                    }
                                    None => {
                                        mapped_values.push(v.clone());
                                    }
                                }
                            }
                            value = Value::Array(mapped_values);
                        }
                        Value::String(_) => {
                            if let Some(mapped_entry) = mapped_entries.get(value.as_str().unwrap())
                            {
                                value = Value::String(mapped_entry.to_string());
                            };
                        }
                        _ => (),
                    }
                }
                if let Some(operations) = unit_transformation_operations.get(k) {
                    if let Value::Number(num) = &value {
                        value = Value::Number(
                            serde_json::value::Number::from_f64(
                                self.calculate_value_units(num.as_f64().unwrap(), operations),
                            )
                            .unwrap(),
                        );
                    }
                }
                transformed_data.insert(key, value);
            }

            transformed_data_set.push(Value::Object(transformed_data));
        }

        Box::new(Self::new(
            serde_json::to_string(&Value::Array(transformed_data_set)).unwrap(),
        ))
    }
}
