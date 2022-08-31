use crate::data_set::DataSet;
use crate::errors::GenericError;
use crate::transformator::Operation;
use oca_rust::state::oca::OCA;
use serde::{Serialize, Serializer};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct CSVDataSet {
    pub plain: String,
}

impl Serialize for CSVDataSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.plain)
    }
}

impl DataSet for CSVDataSet {
    fn new(plain: String) -> Self {
        Self { plain }
    }

    fn load(&self, attribute_types: BTreeMap<String, String>) -> Vec<Value> {
        let mut rows_value = vec![];
        for line in self.plain.lines() {
            rows_value.push(Value::Array(
                line.split(',')
                    .map(|el| Value::String(el.to_string()))
                    .collect(),
            ))
        }
        let mut result = vec![];

        let header_row_v = rows_value.get(0).unwrap().as_array().unwrap();
        let header_row = header_row_v
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect::<Vec<&str>>();

        for row_value in &rows_value[1..rows_value.len()] {
            if let Value::Array(row_v) = row_value {
                let row: Vec<Value> = row_v
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        let attribute_type = attribute_types
                            .get(&header_row.get(i).unwrap().to_string())
                            .unwrap();
                        Self::parse_value(v, attribute_type).unwrap()
                    })
                    .collect();
                result.push(header_row.iter().cloned().zip(row.clone()).collect());
            }
        }
        result
    }

    fn transform_schema(&self, mappings: BTreeMap<String, String>) -> Box<dyn DataSet> {
        let mut plain = self.plain.clone();
        let header_op = plain.lines().take(1).next();
        if let Some(header) = header_op {
            let mut new_header = header.to_string();
            for (k, v) in mappings {
                new_header = header.replace(k.as_str(), v.as_str());
            }
            plain = plain.replace(header, new_header.as_str());
        }
        Box::new(Self::new(plain))
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

impl CSVDataSet {
    fn parse_value(value: &Value, attribute_type: &str) -> Result<Value, GenericError> {
        if value.is_string() {
            let value_str = value.as_str().unwrap().to_string();
            let parsed_value = match attribute_type {
                "Text" => Value::String(value_str),
                "Array[Text]" => {
                    let mut parsed = vec![];
                    for v in serde_json::from_str::<Value>(value_str.replace('|', ",").as_str())?
                        .as_array()
                        .ok_or_else(|| {
                            GenericError::from(format!("{} value is not an array", value_str))
                        })?
                    {
                        parsed.push(Self::parse_value(v, "Text")?)
                    }
                    Value::Array(parsed)
                }
                "Numeric" => Value::Number(value_str.parse()?),
                "Array[Numeric]" => {
                    let mut parsed = vec![];
                    for v in serde_json::from_str::<Value>(value_str.replace('|', ",").as_str())?
                        .as_array()
                        .ok_or_else(|| {
                            GenericError::from(format!("{} value is not an array", value_str))
                        })?
                    {
                        parsed.push(Self::parse_value(v, "Numeric")?)
                    }
                    Value::Array(parsed)
                }
                "Boolean" => Value::Bool(value_str.parse()?),
                "Array[Boolean]" => {
                    let mut parsed = vec![];
                    for v in serde_json::from_str::<Value>(value_str.replace('|', ",").as_str())?
                        .as_array()
                        .ok_or_else(|| {
                            GenericError::from(format!("{} value is not an array", value_str))
                        })?
                    {
                        parsed.push(Self::parse_value(v, "Boolean")?)
                    }
                    Value::Array(parsed)
                }
                "Date" => Value::String(value_str),
                "Array[Date]" => {
                    let mut parsed = vec![];
                    for v in serde_json::from_str::<Value>(value_str.replace('|', ",").as_str())?
                        .as_array()
                        .ok_or_else(|| {
                            GenericError::from(format!("{} value is not an array", value_str))
                        })?
                    {
                        parsed.push(Self::parse_value(v, "Date")?)
                    }
                    Value::Array(parsed)
                }
                _ => Value::Null,
            };

            return Ok(parsed_value);
        }

        Ok(value.clone())
    }
}
