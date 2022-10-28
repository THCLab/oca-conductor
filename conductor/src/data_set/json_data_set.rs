use crate::data_set::DataSet;
use crate::errors::GenericError;
use serde::{Serialize, Serializer};
use std::collections::BTreeMap;
use serde_json::Value;

#[cfg(feature = "transformer")]
use crate::transformer::data_set_transformer::Operation;
#[cfg(feature = "transformer")]
use oca_rust::state::oca::OCA;
#[cfg(feature = "transformer")]
use serde_json::Map;

#[derive(Clone)]
pub struct JSONDataSet {
    pub raw: String,
}

impl Serialize for JSONDataSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.raw)
    }
}

impl DataSet for JSONDataSet {
    fn new(raw: String) -> Box<Self> {
        Box::new(Self { raw })
    }

    fn get_raw(&self) -> String {
        self.raw.clone()
    }

    fn load(&self, _attribute_types: BTreeMap<String, String>) -> Result<Vec<Value>, Vec<GenericError>> {
        match serde_json::from_str(&self.raw).unwrap_or(Value::Null) {
            Value::Array(data_set_array) => Ok(data_set_array),
            Value::Object(data_set_object) => Ok(vec![Value::Object(data_set_object)]),
            _ => {
                Ok(vec![])
            }
        }
    }

    #[cfg(feature = "transformer")]
    fn transform_schema(
        &self,
        _mappings: BTreeMap<String, String>,
        _subset_attributes_op: Option<Vec<String>>,
    ) -> Result<Box<dyn DataSet>, GenericError> {
        // TODO
        Ok(Self::new(self.raw.clone()))
    }

    #[cfg(feature = "transformer")]
    fn transform_data(
        &self,
        oca: &OCA,
        entry_code_mappings: BTreeMap<String, BTreeMap<String, String>>,
        unit_transformation_operations: BTreeMap<String, Vec<Operation>>,
    ) -> Result<Box<dyn DataSet>, Vec<GenericError>> {
        // TODO
        let mut transformed_data_set = vec![];

        for record in self.load(oca.capture_base.attributes.clone())? {
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

        Ok(
            Self::new(
                serde_json::to_string(&Value::Array(transformed_data_set)).unwrap(),
            )
        )
    }
}
