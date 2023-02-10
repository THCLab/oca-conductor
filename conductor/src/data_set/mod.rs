pub mod csv_data_set;
pub mod json_data_set;

use crate::errors::GenericError;
#[cfg(feature = "transformer")]
use crate::transformer::data_set_transformer::{OpType, Operation};
pub use csv_data_set::CSVDataSet;
pub use json_data_set::JSONDataSet;
#[cfg(feature = "transformer")]
use oca_rs::state::oca::OCA;
use serde_json::Value;
use std::collections::BTreeMap;

erased_serde::serialize_trait_object!(DataSet);
dyn_clone::clone_trait_object!(DataSet);

pub trait DataSet: erased_serde::Serialize + dyn_clone::DynClone {
    fn new(raw: String) -> Box<Self>
    where
        Self: Sized;
    fn load(
        &self,
        attribute_types: BTreeMap<String, String>,
    ) -> Result<Vec<Value>, Vec<GenericError>>;
    #[cfg(feature = "transformer")]
    fn transform_schema(
        &self,
        mappings: BTreeMap<String, String>,
        subset_attributes_op: Option<Vec<String>>,
    ) -> Result<Box<dyn DataSet + Sync + Send>, GenericError>;
    #[cfg(feature = "transformer")]
    fn transform_data(
        &self,
        oca: &OCA,
        entry_code_mappings: BTreeMap<String, BTreeMap<String, String>>,
        unit_transformation_operations: BTreeMap<String, Vec<Operation>>,
    ) -> Result<Box<dyn DataSet + Sync + Send>, Vec<GenericError>>;

    fn get_raw(&self) -> String;

    #[cfg(feature = "transformer")]
    fn calculate_value_units(&self, value: f64, operations: &[Operation]) -> f64 {
        let mut result = value;
        for operation in operations {
            result = self.apply_operation(result, operation);
        }

        result
    }

    #[cfg(feature = "transformer")]
    fn apply_operation(&self, value: f64, operation: &Operation) -> f64 {
        match operation.op {
            OpType::Multiply => value * operation.value,
            OpType::Divide => value / operation.value,
            OpType::Add => value + operation.value,
            OpType::Subtract => value - operation.value,
        }
    }
}
