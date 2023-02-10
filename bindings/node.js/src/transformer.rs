#![allow(dead_code)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use crate::{DataSetType, DataSetLoaderRouter};
use oca_conductor::data_set::{DataSet, CSVDataSet};
use oca_conductor::Transformer;
use oca_rs::state::oca::OCA;

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
        overlays: Option<Vec<&str>>
    ) -> Result<&Self> {
        match data_set.t {
            DataSetType::CSVDataSet => {
                let mut internal_data_set = CSVDataSet::new(data_set.raw.clone());
                if let Some(delimiter) = &data_set.delimiter {
                    if let Some(sign) = delimiter.chars().collect::<Vec<char>>().first() {
                        internal_data_set.delimiter(*sign);
                    }
                }
                match self.base.add_data_set(internal_data_set, overlays) {
                    Ok(_) => Ok(self),
                    Err(errors) => Err(
                        napi::Error::from_reason(
                            errors.iter().map(|e| e.to_string()).collect::<Vec<String>>().join(",")
                        )
                    )
                }
            }
        }
    }

    #[napi]
    pub fn transform(&mut self, overlays: Vec<&str>) -> Result<&Self> {
        match self.base.transform(overlays) {
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
