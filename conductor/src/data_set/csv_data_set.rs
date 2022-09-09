use crate::data_set::DataSet;
use crate::errors::GenericError;
use serde::{Serialize, Serializer};
use serde_json::Value;
use std::collections::BTreeMap;

#[cfg(feature = "transformer")]
use crate::transformer::data_set_transformer::Operation;
#[cfg(feature = "transformer")]
use oca_rust::state::oca::OCA;
#[cfg(feature = "transformer")]
use serde_json::Map;

#[derive(Clone)]
pub struct CSVDataSet {
    pub raw: String,
    delimiter: char,
}

impl Serialize for CSVDataSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.raw)
    }
}

impl DataSet for CSVDataSet {
    fn new(raw: String) -> Box<Self> {
        Box::new(Self {
            raw,
            delimiter: ';',
        })
    }

    fn get_raw(&self) -> String {
        self.raw.clone()
    }

    fn load(
        &self,
        attribute_types: BTreeMap<String, String>,
    ) -> Result<Vec<Value>, Vec<GenericError>> {
        let mut errors = vec![];
        let mut rows_value = vec![];
        for line in self.raw.lines() {
            rows_value.push(Value::Array(
                line.split(self.delimiter)
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
                        let attribute_name = header_row.get(i).unwrap().to_string();
                        let attribute_type_op = attribute_types.get(&attribute_name);
                        match attribute_type_op {
                            Some(attribute_type) => match Self::parse_value(v, attribute_type) {
                                Ok(parsed) => parsed,
                                Err(e) => {
                                    errors.push(GenericError::from(format!(
                                        "{}: {}",
                                        attribute_name, e
                                    )));
                                    Value::Null
                                }
                            },
                            None => v.clone(),
                        }
                    })
                    .collect();
                result.push(header_row.iter().cloned().zip(row.clone()).collect());
            }
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(result)
    }

    #[cfg(feature = "transformer")]
    fn transform_schema(
        &self,
        mappings: BTreeMap<String, String>,
    ) -> Result<Box<dyn DataSet>, GenericError> {
        let mut transformed_raw = self.raw.clone();
        if let Some(header_line) = transformed_raw.lines().take(1).next() {
            let headers = header_line
                .split(self.delimiter)
                .map(|header| match mappings.get(header) {
                    Some(mapping) => mapping,
                    None => header,
                })
                .collect::<Vec<&str>>();
            transformed_raw = headers.join(&self.delimiter.to_string())
                + "\n"
                + &transformed_raw
                    .lines()
                    .skip(1)
                    .collect::<Vec<&str>>()
                    .join("\n");
        }
        Ok(Box::new(
            Self::new(transformed_raw).delimiter(self.delimiter),
        ))
    }

    #[cfg(feature = "transformer")]
    fn transform_data(
        &self,
        oca: &OCA,
        entry_code_mappings: BTreeMap<String, BTreeMap<String, String>>,
        unit_transformation_operations: BTreeMap<String, Vec<Operation>>,
    ) -> Result<Box<dyn DataSet>, Vec<GenericError>> {
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

        let mut data = String::new();
        for (i, record_val) in transformed_data_set.iter().enumerate() {
            if let Value::Object(record) = record_val {
                if i == 0 {
                    data.push_str(
                        &record
                            .keys()
                            .map(|v| v.to_string())
                            .collect::<Vec<String>>()
                            .join(&self.delimiter.to_string()),
                    );
                }
                let line = String::from("\n")
                    + &record
                        .values()
                        .map(|v| v.to_string())
                        .collect::<Vec<String>>()
                        .join(&self.delimiter.to_string());
                data.push_str(&line);
            }
        }

        Ok(Box::new(Self::new(data).delimiter(self.delimiter)))
    }
}

impl CSVDataSet {
    pub fn delimiter(&mut self, d: char) -> Self {
        self.delimiter = d;
        self.clone()
    }

    fn parse_value(value: &Value, attribute_type: &str) -> Result<Value, GenericError> {
        if value.is_string() {
            let value_str = value.as_str().unwrap().to_string();
            let parsed_value = match attribute_type {
                "Text" => Value::String(value_str),
                "Array[Text]" => {
                    let mut parsed = vec![];
                    for v in serde_json::from_str::<Value>(value_str.as_str())
                        .unwrap_or_else(|_| Value::String(value_str.clone()))
                        .as_array()
                        .ok_or_else(|| {
                            GenericError::from(format!("\"{}\" value is not an array", value_str))
                        })?
                    {
                        parsed.push(Self::parse_value(v, "Text")?)
                    }
                    Value::Array(parsed)
                }
                "Numeric" => Value::Number(value_str.parse()?),
                "Array[Numeric]" => {
                    let mut parsed = vec![];
                    for v in serde_json::from_str::<Value>(value_str.as_str())
                        .unwrap_or_else(|_| Value::String(value_str.clone()))
                        .as_array()
                        .ok_or_else(|| {
                            GenericError::from(format!("\"{}\" value is not an array", value_str))
                        })?
                    {
                        parsed.push(Self::parse_value(v, "Numeric")?)
                    }
                    Value::Array(parsed)
                }
                "Boolean" => Value::Bool(value_str.parse()?),
                "Array[Boolean]" => {
                    let mut parsed = vec![];
                    for v in serde_json::from_str::<Value>(value_str.as_str())
                        .unwrap_or_else(|_| Value::String(value_str.clone()))
                        .as_array()
                        .ok_or_else(|| {
                            GenericError::from(format!("\"{}\" value is not an array", value_str))
                        })?
                    {
                        parsed.push(Self::parse_value(v, "Boolean")?)
                    }
                    Value::Array(parsed)
                }
                "Date" => Value::String(value_str),
                "Array[Date]" => {
                    let mut parsed = vec![];
                    for v in serde_json::from_str::<Value>(value_str.as_str())
                        .unwrap_or_else(|_| Value::String(value_str.clone()))
                        .as_array()
                        .ok_or_else(|| {
                            GenericError::from(format!("\"{}\" value is not an array", value_str))
                        })?
                    {
                        parsed.push(Self::parse_value(v, "Date")?)
                    }
                    Value::Array(parsed)
                }
                _ => Value::Null,
                // TODO add parsing Binary and SAI types
            };

            return Ok(parsed_value);
        }

        Ok(value.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oca_rust::state::oca::OCA;

    fn setup_oca() -> OCA {
        let common_assets_dir_path = format!("{}/../assets", env!("CARGO_MANIFEST_DIR"));
        let oca_result = oca_zip_resolver::resolve_from_zip(
            format!("{}/oca_bundle.zip", common_assets_dir_path).as_str(),
        );
        assert!(oca_result.is_ok());
        oca_result.unwrap()
    }

    #[test]
    fn load_csv_data_set() {
        let oca = setup_oca();
        let result = CSVDataSet::new(
            r#"asd
test@example.com"#
                .to_string(),
        )
        .load(oca.capture_base.attributes);
        assert!(result.is_ok());
    }

    #[test]
    fn load_csv_data_set_with_custom_delimiter() {
        let oca = setup_oca();
        let result = CSVDataSet::new(
            r#"first,second
1,2"#
                .to_string(),
        )
        .delimiter(',')
        .load(oca.capture_base.attributes);

        assert!(result.is_ok());
        if let Value::Object(record) = result.unwrap().first().unwrap() {
            assert_eq!(record.len(), 2);
        }
    }

    #[test]
    fn parse_vaules_with_invalid_array() {
        let array_types = vec![
            "Array[Text]",
            "Array[Numeric]",
            "Array[Date]",
            "Array[Boolean]",
        ];
        for array_type in array_types {
            let result = CSVDataSet::parse_value(&Value::String("asd".to_string()), array_type);
            assert!(result.is_err());
        }
    }
}
