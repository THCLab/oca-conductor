use oca_rust::state::oca::OCA;
use serde::Serialize;
use serde_json::Value;

mod validator;

pub struct OCAConductor {
    pub oca: OCA,
    validator: validator::OCAValidator,
    constraints_config: Option<ConstraintsConfig>,
    pub data_sets: Vec<Value>,
}

#[derive(Clone)]
pub struct ConstraintsConfig {
    pub fail_on_additional_attributes: bool,
}

#[derive(Serialize)]
pub struct ValidationResult {
    pub success: bool,
    pub errors: Vec<String>,
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationResult {
    pub fn new() -> ValidationResult {
        ValidationResult {
            success: true,
            errors: vec![],
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.success = false;
        self.errors.push(error);
    }

    pub fn add_errors(&mut self, errors: Vec<String>) {
        self.success = false;
        self.errors.extend(errors);
    }
}

impl OCAConductor {
    pub fn load_oca(oca: OCA) -> OCAConductor {
        let validator = validator::OCAValidator::new(&oca);
        OCAConductor {
            oca,
            validator,
            constraints_config: None,
            data_sets: vec![],
        }
    }

    pub fn set_constraints(&mut self, config: ConstraintsConfig) {
        self.validator.set_constraints(config.clone());
        self.constraints_config = Some(config);
    }

    pub fn add_data_set(&mut self, data_set: &str) {
        let mut data_set_value = serde_json::from_str(data_set).unwrap_or(Value::Null);
        match data_set_value {
            Value::Array(ref mut data_set_array) => self.data_sets.append(data_set_array),
            Value::Object(ref _data_set_object) => self.data_sets.push(data_set_value),
            _ => {}
        }
    }

    pub fn validate(&self) -> ValidationResult {
        let mut result = ValidationResult::new();

        if let Err(errors) = self.validator.validate(self.data_sets.clone()) {
            result.add_errors(errors);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_oca() -> OCA {
        let common_assets_dir_path = format!("{}/../assets", env!("CARGO_MANIFEST_DIR"));
        let oca_result = zip_resolver::resolve_from_zip(
            format!("{}/oca_bundle.zip", common_assets_dir_path).as_str(),
        );
        assert!(oca_result.is_ok());
        oca_result.unwrap()
    }

    #[test]
    fn validation_of_proper_data_set_should_return_successful_validation_result() {
        let oca = setup_oca();
        let mut oca_conductor = OCAConductor::load_oca(oca);
        oca_conductor.add_data_set(
            r#"[
                {
                    "email*": "test@example.com",
                    "licenses*": ["A"],
                    "number": 24,
                    "numbers": [22, "23"],
                    "date": "01.01.1999",
                    "dates": ["01.01.2000"],
                    "bool": true,
                    "bools": [false, true]
                }
            ]"#,
        );
        let validation_result = oca_conductor.validate();
        assert!(validation_result.success);
    }
}
