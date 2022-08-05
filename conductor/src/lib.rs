use oca_rust::state::oca::{DynOverlay, OCA};
use said::derivation::SelfAddressing;
use serde_json::Value;

pub mod data_set_loader;
use data_set_loader::DataSetLoader;
pub mod transformation_result;
mod transformator;
pub mod validation_result;
mod validator;
use transformation_result::TransformationResult;
use validation_result::ValidationResult;

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

    pub fn add_data_set<T>(&mut self, data_set_loader: T)
    where
        T: DataSetLoader,
    {
        self.data_sets.extend(data_set_loader.load());
    }

    pub fn validate(&self) -> ValidationResult {
        let mut result = ValidationResult::new();

        if let Err(errors) = self.validator.validate(self.data_sets.clone()) {
            result.add_errors(errors);
        }

        result
    }

    pub fn transform_data(&self, overlays: Vec<&str>) -> TransformationResult {
        let mut transformation_result = TransformationResult::new();

        let mut additional_overlays: Vec<DynOverlay> = vec![];

        let cb_json = serde_json::to_string(&self.oca.capture_base).unwrap();
        let oca_cb_sai = format!("{}", SelfAddressing::Blake3_256.derive(cb_json.as_bytes()));
        for (i, overlay_str) in overlays.iter().enumerate() {
            match serde_json::from_str::<DynOverlay>(overlay_str) {
                Ok(mut overlay) => {
                    if oca_cb_sai.eq(overlay.capture_base()) {
                        additional_overlays.push(overlay);
                    } else {
                        transformation_result.add_error(format!(
                            "Overlay at position {}: Incompatible with OCA Capture Base.",
                            i + 1
                        ))
                    }
                }
                Err(_) => transformation_result.add_error(format!(
                    "Overlay at position {}: Parsing failed. Invalid format.",
                    i + 1
                )),
            }
        }

        if !transformation_result.success {
            return transformation_result;
        }

        let result = transformator::transform_data(
            &self.oca.overlays,
            additional_overlays,
            self.data_sets.clone(),
        );

        transformation_result.add_data_sets(result);

        transformation_result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_set_loader::json_data_set::JSONDataSet;

    fn setup_oca() -> OCA {
        let common_assets_dir_path = format!("{}/../assets", env!("CARGO_MANIFEST_DIR"));
        let oca_result = zip_resolver::resolve_from_zip(
            format!("{}/oca_bundle.zip", common_assets_dir_path).as_str(),
        );
        assert!(oca_result.is_ok());
        oca_result.unwrap()
    }

    #[test]
    fn transform_data_with_attribute_mapping_overlay() {
        let oca = setup_oca();
        let mut oca_conductor = OCAConductor::load_oca(oca);
        oca_conductor.add_data_set(JSONDataSet::new(
            r#"[
                    {
                        "e-mail*": "test@example.com"
                    }
                ]"#,
        ));
        let transformation_result = oca_conductor.transform_data(vec![
            r#"
{
    "capture_base":"EKmZWuURpiUdl_YAMGQbLiossAntKt1DJ0gmUMYSz7Yo",
    "type":"spec/overlays/mapping/1.0",
    "attr_mapping": {
        "email*":"e-mail*"
    }
}
              "#,
        ]);

        assert!(transformation_result
            .result
            .unwrap()
            .get(0)
            .unwrap()
            .get("email*")
            .is_some())
    }

    #[test]
    fn transform_data_with_entry_code_mapping_overlay() {
        let oca = setup_oca();
        let mut oca_conductor = OCAConductor::load_oca(oca);
        oca_conductor.add_data_set(JSONDataSet::new(
            r#"[
                    {
                        "licenses*": ["a"]
                    }
                ]"#,
        ));
        let transformation_result = oca_conductor.transform_data(vec![
            r#"
{
    "capture_base":"EKmZWuURpiUdl_YAMGQbLiossAntKt1DJ0gmUMYSz7Yo",
    "type":"spec/overlays/entry_code_mapping/1.0",
    "attr_mapping": {
        "licenses*": [
            "a:A","b:B","c:C","d:D","e:E"
        ]
    }
}
              "#,
        ]);

        assert_eq!(
            transformation_result
                .result
                .unwrap()
                .get(0)
                .unwrap()
                .get("licenses*")
                .unwrap()
                .get(0)
                .unwrap(),
            "A"
        );
    }

    #[test]
    fn transform_data_with_invalid_overlay() {
        let oca = setup_oca();
        let mut oca_conductor = OCAConductor::load_oca(oca);
        oca_conductor.add_data_set(JSONDataSet::new(
            r#"[
                    {
                        "e-mail*": "test@example.com"
                    }
                ]"#,
        ));
        let transformation_result = oca_conductor.transform_data(vec![r#"invalid"#]);

        assert!(!transformation_result.success)
    }

    #[test]
    fn validation_of_proper_data_set_should_return_successful_validation_result() {
        let oca = setup_oca();
        let mut oca_conductor = OCAConductor::load_oca(oca);
        oca_conductor.add_data_set(JSONDataSet::new(
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
        ));
        let validation_result = oca_conductor.validate();
        assert!(validation_result.success);
    }
}
