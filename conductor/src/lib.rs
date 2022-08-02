use oca_rust::state::oca::OCA;
use oca_rust::state::oca::DynOverlay;
use oca_rust::state::oca::overlay;
use serde::Serialize;
use serde_json::Value;
use serde_json::Map;
use std::collections::BTreeMap;

mod validator;

pub struct OCAConductor {
    pub oca: OCA,
    validator: validator::OCAValidator,
    constraints_config: Option<ConstraintsConfig>,
    pub data_sets: Vec<Value>,
}

#[derive(Clone)]
pub struct ConstraintsConfig {
    pub fail_on_additional_attributes: bool,
}

#[derive(Serialize)]
pub struct ValidationResult {
    pub success: bool,
    pub errors: Vec<String>,
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationResult {
    pub fn new() -> ValidationResult {
        ValidationResult {
            success: true,
            errors: vec![],
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.success = false;
        self.errors.push(error);
    }

    pub fn add_errors(&mut self, errors: Vec<String>) {
        self.success = false;
        self.errors.extend(errors);
    }
}

impl OCAConductor {
    pub fn load_oca(oca: OCA) -> OCAConductor {
        let validator = validator::OCAValidator::new(&oca);
        OCAConductor {
            oca,
            validator,
            constraints_config: None,
            data_sets: vec![],
        }
    }

    pub fn set_constraints(&mut self, config: ConstraintsConfig) {
        self.validator.set_constraints(config.clone());
        self.constraints_config = Some(config);
    }

    pub fn add_data_set(&mut self, data_set: &str) {
        let mut data_set_value = serde_json::from_str(data_set).unwrap_or(Value::Null);
        match data_set_value {
            Value::Array(ref mut data_set_array) => self.data_sets.append(data_set_array),
            Value::Object(ref _data_set_object) => self.data_sets.push(data_set_value),
            _ => {}
        }
    }

    pub fn validate(&self) -> ValidationResult {
        let mut result = ValidationResult::new();

        if let Err(errors) = self.validator.validate(self.data_sets.clone()) {
            result.add_errors(errors);
        }

        result
    }

    pub fn transform_data(&self, overlays: Vec<&str>) -> Vec<Value> {
        let mut transformed_data_set = vec![];

        let mut attribute_mappings: Option<BTreeMap<&String, &String>> = None;
        let mut entry_code_mappings: Option<BTreeMap<String, BTreeMap<&str, &str>>> = None;

        let mut additional_overlays: Vec<DynOverlay> = vec![];

        for overlay_str in overlays {
            additional_overlays.push(serde_json::from_str(overlay_str).unwrap());
        }

        for overlay in &additional_overlays {
            if attribute_mappings.is_none() && overlay.overlay_type().contains("/mapping/") {
                let ov = overlay
                    .as_any()
                    .downcast_ref::<overlay::AttributeMapping>()
                    .unwrap();
                let keys = ov.attr_mapping.keys().clone();
                let values = ov.attr_mapping.values().clone();
                attribute_mappings = Some(values.into_iter().zip(keys.into_iter()).collect());
            } else if entry_code_mappings.is_none() && overlay.overlay_type().contains("/entry_code_mapping/") {
                let ov = overlay
                    .as_any()
                    .downcast_ref::<overlay::EntryCodeMapping>()
                    .unwrap();
                let mut entry_code_mappings_tmp = BTreeMap::new();

                for (attr_name, value) in &ov.attr_mapping {
                    let mut mappings = BTreeMap::new();
                    for v in value {
                        let splitted = v.split(':').collect::<Vec<&str>>();
                        mappings.insert(*splitted.get(0).unwrap(), *splitted.get(1).unwrap());
                    }
                    entry_code_mappings_tmp.insert(attr_name.clone(), mappings);
                }
                entry_code_mappings = Some(entry_code_mappings_tmp);
            }
        }

        for overlay in &self.oca.overlays {
            if attribute_mappings.is_none() && overlay.overlay_type().contains("/mapping/") {
                let ov = overlay
                    .as_any()
                    .downcast_ref::<overlay::AttributeMapping>()
                    .unwrap();
                let keys = ov.attr_mapping.keys().clone();
                let values = ov.attr_mapping.values().clone();
                attribute_mappings = Some(values.into_iter().zip(keys.into_iter()).collect());
            } else if entry_code_mappings.is_none() && overlay.overlay_type().contains("/entry_code_mapping/") {
                let ov = overlay
                    .as_any()
                    .downcast_ref::<overlay::EntryCodeMapping>()
                    .unwrap();
                let mut entry_code_mappings_tmp = BTreeMap::new();

                for (attr_name, value) in &ov.attr_mapping {
                    let mut mappings = BTreeMap::new();
                    for v in value {
                        let splitted = v.split(':').collect::<Vec<&str>>();
                        mappings.insert(*splitted.get(0).unwrap(), *splitted.get(1).unwrap());
                    }
                    entry_code_mappings_tmp.insert(attr_name.clone(), mappings);
                }
                entry_code_mappings = Some(entry_code_mappings_tmp);
            }
        }

        for data_set in &self.data_sets {
            let data_set_map = data_set.as_object().unwrap();
            let mut transformed_data = Map::new();
            for (k, v) in data_set_map {
                let mut key = k.to_string();
                let mut value = v.clone();
                if let Some(ref mappings) = attribute_mappings {
                    if let Some(mapped_key) = mappings.get(k) {
                        key = mapped_key.to_string();
                    }
                }
                if let Some(ref mappings) = entry_code_mappings {
                    if let Some(mapped_entries) = mappings.get(k) {
                        match value {
                            Value::Array(ref values_vec) => {
                                let mut mapped_values = vec![];
                                for v in values_vec {
                                    match mapped_entries.get(v.as_str().unwrap()) {
                                      Some(mapped_entry) => {
                                        mapped_values.push(Value::String(mapped_entry.to_string()));
                                      },
                                      None => {
                                        mapped_values.push(v.clone());
                                      }
                                    }
                                }
                                value = Value::Array(mapped_values);
                            },
                            Value::String(_) => {
                                if let Some(mapped_entry) = mapped_entries.get(value.as_str().unwrap()) {
                                    value = Value::String(mapped_entry.to_string());
                                };
                            },
                            _ => {}
                        }

                    }
                }
                transformed_data.insert(key, value);
            }

            transformed_data_set.push(Value::Object(transformed_data));
        }
        transformed_data_set
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_oca() -> OCA {
        let common_assets_dir_path = format!("{}/../assets", env!("CARGO_MANIFEST_DIR"));
        let oca_result = zip_resolver::resolve_from_zip(
            format!("{}/oca_bundle.zip", common_assets_dir_path).as_str(),
        );
        assert!(oca_result.is_ok());
        oca_result.unwrap()
    }

    #[test]
    fn transform_data() {
        let oca = setup_oca();
        println!(
            "{}", serde_json::to_string_pretty(&oca).unwrap()
        );
        let mut oca_conductor = OCAConductor::load_oca(oca);
        oca_conductor.add_data_set(
            r#"[
                {
                    "e-mail*": "test@example.com",
                    "licenses*": ["a"],
                    "number": 24,
                    "numbers": [22, "23"],
                    "date": "01.01.1999",
                    "dates": ["01.01.2000"],
                    "bool": true,
                    "bools": [false, true]
                }
            ]"#,
        );
        let data = oca_conductor.transform_data(
            vec![
              r#"
{
    "capture_base":"EKmZWuURpiUdl_YAMGQbLiossAntKt1DJ0gmUMYSz7Yo",
    "type":"spec/overlays/mapping/1.0",
    "attr_mapping": {
        "email*":"e-mail*"
    }
}
              "#,
              r#"
{
    "capture_base":"EKmZWuURpiUdl_YAMGQbLiossAntKt1DJ0gmUMYSz7Yo",
    "type":"spec/overlays/entry_code_mapping/1.0",
    "attr_mapping": {
        "licenses*": [
            "a:S","b:B","c:C","d:D","e:E"
        ]
    }
}
              "#
            ]
        );
        println!(
            "{}", serde_json::to_string_pretty(&data).unwrap()
        )
    }

    #[test]
    fn validation_of_proper_data_set_should_return_successful_validation_result() {
        let oca = setup_oca();
        let mut oca_conductor = OCAConductor::load_oca(oca);
        oca_conductor.add_data_set(
            r#"[
                {
                    "email*": "test@example.com",
                    "licenses*": ["A"],
                    "number": 24,
                    "numbers": [22, "23"],
                    "date": "01.01.1999",
                    "dates": ["01.01.2000"],
                    "bool": true,
                    "bools": [false, true]
                }
            ]"#,
        );
        let validation_result = oca_conductor.validate();
        assert!(validation_result.success);
    }
}
