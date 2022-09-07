#![allow(dead_code)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use crate::{DataSetType, DataSetLoaderRouter};
use oca_conductor::data_set::{DataSet, CSVDataSet};
use oca_conductor::Transformer;
use oca_rust::state::oca::OCA;

#[napi(js_name = "Transformer")]
pub struct TransformerWrapper {
    base: Transformer,
}

#[napi]
impl TransformerWrapper {
    #[napi(constructor)]
    pub fn new(env: Env, oca: napi::JsObject) -> Result<Self> {
        let base = Transformer::new(env.from_js_value::<OCA, napi::JsObject>(oca)?);
        Ok(Self { base })
    }

    #[napi]
    pub fn add_data_set(
        &mut self,
        #[napi(ts_arg_type = "CSVDataSet")] data_set: &DataSetLoaderRouter,
    ) -> &Self {
        match data_set.t {
            DataSetType::CSVDataSet => {
                self.base.add_data_set(CSVDataSet::new(data_set.raw.clone()));
            }
        }
        self
    }

    #[napi]
    pub fn transform_pre(&mut self, overlays: Vec<&str>) -> Result<&Self> {
        match self.base.transform_pre(overlays) {
            Ok(_) => Ok(self),
            Err(errors) => Err(
                napi::Error::from_reason(
                    errors.iter().map(|e| e.to_string()).collect::<Vec<String>>().join(",")
                )
            )
        }
    }

    #[napi]
    pub fn get_raw_datasets(&self) -> Vec<String> {
        self.base.get_raw_datasets()
    }
}
