use crate::data_set::DataSet;
use serde::Serialize;

#[derive(Serialize)]
pub struct TransformationResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Vec<Box<dyn DataSet>>>,
}

impl Default for TransformationResult {
    fn default() -> Self {
        Self::new()
    }
}

impl TransformationResult {
    pub fn new() -> Self {
        Self {
            success: true,
            errors: None,
            result: None,
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.success = false;
        match std::mem::take(&mut self.errors) {
            Some(mut e) => {
                e.push(error);
            }
            None => {
                self.errors = Some(vec![error]);
            }
        }
    }

    pub fn add_errors(&mut self, errors: Vec<String>) {
        self.success = false;
        match std::mem::take(&mut self.errors) {
            Some(mut e) => e.extend(errors),
            None => {
                self.errors = Some(errors);
            }
        }
    }

    pub fn add_data_sets(&mut self, data_sets: Vec<Box<dyn DataSet>>) {
        self.result = Some(data_sets);
    }
}
