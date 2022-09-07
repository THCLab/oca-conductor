use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::{DataSetType, DataSetLoaderRouter};
use oca_conductor::data_set::{DataSet, CSVDataSet};
use oca_conductor::errors::GenericError;
use oca_conductor::validator::ConstraintsConfig;
use oca_conductor::Validator;
use oca_rust::state::oca::OCA;

#[napi(js_name = "Validator")]
pub struct ValidatorWrapper {
    base: Validator,
}

#[napi]
impl ValidatorWrapper {
    #[napi(constructor)]
    pub fn new(env: Env, oca: napi::JsObject) -> Result<Self> {
        let base = Validator::new(env.from_js_value::<OCA, napi::JsObject>(oca)?);
        Ok(Self { base })
    }

    #[napi]
    pub fn set_constraints(
        &mut self,
        #[napi(ts_arg_type = "ConstraintsConfig")] config: ConstraintsConfigWrapper,
    ) {
        self.base.set_constraints(config.to_base());
    }

    #[napi]
    pub fn add_data_set(
        &mut self,
        #[napi(ts_arg_type = "CSVDataSet")] data_set: &DataSetLoaderRouter,
    ) {
        match data_set.t {
            DataSetType::CSVDataSet => {
                self.base.add_data_set(CSVDataSet::new(data_set.raw.clone()));
            }
        }
        // self.update_state();
    }

    #[napi]
    pub fn validate(&self) -> ValidationResult {
        ValidationResult::init(self.base.validate())
    }

    #[napi]
    pub fn get_raw_datasets(&self) -> Vec<String> {
        match self.base.validate() {
            Ok(_) => self.base.data_sets.iter().map(|ds| ds.get_raw()).collect(),
            Err(errors) => errors.iter().map(|e| e.to_string()).collect()
        }
    }

    /*
    fn update_state(&mut self) {
        self.data_sets = self.base.data_sets.clone();
    }
    */
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
    pub errors: Option<Vec<String>>,
}

impl ValidationResult {
    fn init(result: std::result::Result<(), Vec<GenericError>>) -> Self {
        match result {
            Ok(_) => Self {
                success: true,
                errors: None,
            },
            Err(errors) => Self {
                success: false,
                errors: Some(errors.iter().map(|e| e.to_string()).collect())
            }
        }
    }
}
