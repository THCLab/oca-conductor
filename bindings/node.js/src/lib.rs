use napi::bindgen_prelude::*;
use napi_derive::napi;
use oca_conductor::ConstraintsConfig;
use oca_conductor::OCAConductor;
use oca_conductor::validation_result::ValidationResult;
use oca_conductor::transformation_result::TransformationResult;
use oca_rust::state::oca::OCA;

#[napi]
pub fn resolve_from_zip(env: Env, path: String) -> Result<napi::JsObject> {
    let resolved = zip_resolver::resolve_from_zip(path.as_str());
    match resolved {
        Ok(oca) => Ok(env.to_js_value(&oca)?.coerce_to_object()?),
        Err(e) => Err(Error::from_reason(e)),
    }
}

#[napi(js_name = "OcaConductor")]
pub struct OcaConductorWrapper {
    base: OCAConductor,
    pub data_sets: Vec<serde_json::Value>,
}

#[napi]
impl OcaConductorWrapper {
    #[napi(factory)]
    pub fn load_oca(env: Env, oca: napi::JsObject) -> Result<Self> {
        let base = OCAConductor::load_oca(env.from_js_value::<OCA, napi::JsObject>(oca)?);
        let data_sets = base.data_sets.clone();
        Ok(Self { base, data_sets })
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
        #[napi(ts_arg_type = "object | object[]")] data_set: serde_json::Value,
    ) {
        self.base
            .add_data_set(serde_json::to_string(&data_set).unwrap().as_str());
        self.update_state();
    }

    #[napi(ts_return_type = "ValidationResult")]
    pub fn validate(&self) -> ValidationResultWrapper {
        ValidationResultWrapper::init(self.base.validate())
    }

    #[napi(ts_return_type = "TransformationResult")]
    pub fn transform_data(&self, overlays: Vec<&str>) -> TransformationResultWrapper {
        TransformationResultWrapper::init(self.base.transform_data(overlays))
    }

    fn update_state(&mut self) {
        self.data_sets = self.base.data_sets.clone();
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

#[napi(object, js_name = "ValidationResult")]
pub struct ValidationResultWrapper {
    pub success: bool,
    pub errors: Vec<String>,
}

impl ValidationResultWrapper {
    fn init(base: ValidationResult) -> Self {
        Self {
            success: base.success,
            errors: base.errors,
        }
    }
}

#[napi(object, js_name = "TransformationResult")]
pub struct TransformationResultWrapper {
    pub success: bool,
    pub errors: Option<Vec<String>>,
    #[napi(ts_type = "Array<object>")]
    pub result: Option<Vec<serde_json::Value>>,
}

impl TransformationResultWrapper {
    fn init(base: TransformationResult) -> Self {
        Self {
            success: base.success,
            errors: base.errors,
            result: base.result,
        }
    }
}
