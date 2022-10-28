use napi::bindgen_prelude::*;
use napi_derive::napi;

use oca_conductor::data_set::{DataSet, JSONDataSet};
use oca_conductor::validator::ConstraintsConfig;
use oca_conductor::validator::ValidationError;
use oca_conductor::Validator;
use oca_rust::state::oca::OCA;

#[napi(js_name = "Validator")]
pub struct ValidatorWrapper {
    base: Validator,
    many: bool
}

#[napi]
impl ValidatorWrapper {
    #[napi(constructor)]
    pub fn new(env: Env, oca: napi::JsObject) -> Result<Self> {
        let base = Validator::new(env.from_js_value::<OCA, napi::JsObject>(oca)?);
        Ok(Self { base, many: false })
    }

    #[napi]
    pub fn set_constraints(
        &mut self,
        #[napi(ts_arg_type = "ConstraintsConfig")] config: ConstraintsConfigWrapper,
    ) -> &Self {
        self.base.set_constraints(config.to_base());
        self
    }

    #[napi]
    pub fn validate(
        &mut self,
        env: Env,
        #[napi(ts_arg_type = "object | object[]")] record: Object
    ) -> ValidationResult {
        let record_val = env.from_js_value::<serde_json::Value, napi::JsObject>(record).unwrap();
        if record_val.is_array() {
            self.many = true
        }
        self.base.add_data_set(JSONDataSet::new(
            serde_json::to_string(&record_val).unwrap()
        ));
        ValidationResult::init(self.base.validate(), self.many)
    }

    #[napi]
    pub fn get_raw_datasets(&self) -> Vec<String> {
        match self.base.validate() {
            Ok(_) => self.base.data_sets.iter().map(|ds| ds.get_raw()).collect(),
            Err(errors) => errors.iter().map(|e| e.to_string()).collect()
        }
    }
}

#[napi(object, js_name = "ConstraintsConfig")]
pub struct ConstraintsConfigWrapper {
    pub fail_on_additional_attributes: bool,
}

impl ConstraintsConfigWrapper {
    fn to_base(&self) -> ConstraintsConfig {
        ConstraintsConfig {
            fail_on_additional_attributes: self.fail_on_additional_attributes,
        }
    }
}

#[napi(object)]
pub struct ValidationResult {
    pub success: bool,
    pub errors: Option<serde_json::Value>,
}

impl ValidationResult {
    fn init(result: std::result::Result<(), Vec<ValidationError>>, many_records: bool) -> Self {
        match result {
            Ok(_) => Self {
                success: true,
                errors: None,
            },
            Err(errors) => Self {
                success: false,
                errors: Some(Self::format_errors(errors, many_records))
            }
        }
    }

    fn format_errors(errors: Vec<ValidationError>, many_records: bool) -> serde_json::Value {
        let mut result_map: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
        for error in errors {
            if many_records {
                match result_map.get_mut(&error.record) {
                    Some(record_errors) => {
                        if let serde_json::Value::Object(record_errors_map) = record_errors {
                            record_errors_map.insert(error.attribute_name, serde_json::Value::String(error.message));
                        }
                    },
                    None => {
                        let mut record_errors_map = serde_json::Map::new();
                        record_errors_map.insert(error.attribute_name, serde_json::Value::String(error.message));
                        result_map.insert(error.record, serde_json::Value::Object(record_errors_map));
                    }
                }
            } else {
                result_map.insert(error.attribute_name, serde_json::Value::String(error.message));
            }
        }
        serde_json::Value::Object(result_map)
    }
}
