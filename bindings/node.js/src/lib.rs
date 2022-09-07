#![allow(dead_code)]

use napi::bindgen_prelude::*;
use napi_derive::napi;

#[cfg(feature = "transformer")]
pub mod transformer;
#[cfg(feature = "validator")]
pub mod validator;

#[napi]
pub fn resolve_from_zip(env: Env, path: String) -> Result<napi::JsObject> {
    let resolved = oca_zip_resolver::resolve_from_zip(path.as_str());
    match resolved {
        Ok(oca) => Ok(env.to_js_value(&oca)?.coerce_to_object()?),
        Err(e) => Err(Error::from_reason(e)),
    }
}

pub enum DataSetType {
    CSVDataSet,
}

#[napi]
pub struct DataSetLoaderRouter {
    t: DataSetType,
    raw: String,
}

#[napi(js_name = "CSVDataSet")]
pub struct CSVDataSetWrapper {
    raw: String,
}

#[napi]
impl CSVDataSetWrapper {
    #[napi(constructor)]
    pub fn new(
        data_set: String,
    ) -> DataSetLoaderRouter {
        DataSetLoaderRouter {
            t: DataSetType::CSVDataSet,
            raw: data_set,
        }
    }
}
