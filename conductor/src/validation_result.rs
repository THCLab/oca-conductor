use serde::Serialize;

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
