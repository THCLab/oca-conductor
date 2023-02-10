pub use oca_conductor::data_set::DataSet;
pub use oca_conductor::data_set::JSONDataSet;
pub use oca_conductor::errors::GenericError;
pub use oca_conductor::validator::{ConstraintsConfig, Validator as ValidatorRaw};

use oca_rust::controller::load_oca;
use std::sync::RwLock;

#[derive(Debug)]
pub enum ValidationError {
    Other {
        data_set: String,
        record: String,
        attribute_name: String,
        message: String,
    },
}

#[derive(Debug)]
pub enum ValidationErrors {
    List { errors: Vec<ValidationError> },
}

impl std::error::Error for ValidationErrors {}
impl std::fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &*self {
            ValidationErrors::List { errors } => write!(f, "{:?}", errors),
        }
    }
}

impl std::error::Error for ValidationError {}
impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &*self {
            ValidationError::Other {
                data_set,
                record,
                attribute_name,
                message,
            } => write!(
                f,
                "Data Set: {}, Record {}: '{}' {}",
                data_set, record, attribute_name, message
            ),
        }
    }
}

pub struct Validator {
    base: RwLock<ValidatorRaw>,
}

impl Validator {
    pub fn new(oca: String) -> Self {
        let oca = load_oca(&mut oca.as_bytes()).unwrap().finalize();
        let base = ValidatorRaw::new(oca);
        Self {
            base: RwLock::new(base),
        }
    }

    pub fn set_constraints(&self, config: ConstraintsConfig) {
        self.base.write().unwrap().set_constraints(config);
    }

    pub fn validate(&self, record: String) -> Result<(), ValidationErrors> {
        let record_val: serde_json::Value = serde_json::from_str(&record).unwrap();
        self.base.write().unwrap().add_data_set(JSONDataSet::new(
            serde_json::to_string(&record_val).unwrap(),
        ));
        let r = self.base.read().unwrap().validate();

        match r {
            Ok(_) => Ok(()),
            Err(errors) => {
                return Err(ValidationErrors::List {
                    errors: errors
                        .iter()
                        .map(|e| ValidationError::Other {
                            data_set: e.data_set.clone(),
                            record: e.record.clone(),
                            attribute_name: e.attribute_name.clone(),
                            message: e.message.clone(),
                        })
                        .collect(),
                });
            }
        }
    }
}

uniffi::include_scaffolding!("validator");
