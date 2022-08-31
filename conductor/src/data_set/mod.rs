pub mod csv_data_set;
pub mod json_data_set;

use crate::transformator::{OpType, Operation};
use oca_rust::state::oca::OCA;
use serde_json::Value;
use std::collections::BTreeMap;

erased_serde::serialize_trait_object!(DataSet);
dyn_clone::clone_trait_object!(DataSet);

pub trait DataSet: erased_serde::Serialize + dyn_clone::DynClone {
    fn new(plain: String) -> Self
    where
        Self: Sized;
    fn load(&self, attribute_types: BTreeMap<String, String>) -> Vec<Value>;
    fn transform_schema(&self, mappings: BTreeMap<String, String>) -> Box<dyn DataSet>;
    fn transform_data(
        &self,
        oca: &OCA,
        entry_code_mappings: BTreeMap<String, BTreeMap<String, String>>,
        unit_transformation_operations: BTreeMap<String, Vec<Operation>>,
    ) -> Box<dyn DataSet>;

    fn calculate_value_units(&self, value: f64, operations: &[Operation]) -> f64 {
        let mut result = value;
        for operation in operations {
            result = self.apply_operation(result, operation);
        }

        result
    }

    fn apply_operation(&self, value: f64, operation: &Operation) -> f64 {
        match operation.op {
            OpType::Multiply => value * operation.value,
            OpType::Divide => value / operation.value,
            OpType::Add => value + operation.value,
            OpType::Subtract => value - operation.value,
        }
    }
}
