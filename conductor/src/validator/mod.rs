use crate::errors::GenericError;
use oca_rust::state::{attribute::AttributeType, entry_codes::EntryCodes, oca::overlay, oca::OCA};
use regex::Regex;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

mod attribute_validator;
use attribute_validator::AttributeValidator;

use crate::data_set::DataSet;

pub struct Validator {
    pub data_sets: Vec<Box<dyn DataSet>>,
    constraints_config: Option<ConstraintsConfig>,
    attribute_validators: HashMap<String, AttributeValidator>,
    attribute_types: BTreeMap<String, String>,
}

#[derive(Clone)]
pub struct ConstraintsConfig {
    pub fail_on_additional_attributes: bool,
}

impl Validator {
    pub fn new(oca: OCA) -> Self {
        Self {
            data_sets: vec![],
            constraints_config: None,
            attribute_validators: Self::parse_oca_attributes_to_validators(&oca),
            attribute_types: oca.capture_base.attributes.clone(),
        }
    }

    pub fn set_constraints(&mut self, config: ConstraintsConfig) {
        self.constraints_config = Some(config);
    }

    pub fn add_data_set(&mut self, data_set: Box<dyn DataSet>) -> &mut Self {
        self.data_sets.push(data_set);
        self
    }

    pub fn validate(&self) -> Result<(), Vec<GenericError>> {
        let mut validation_errors: Vec<GenericError> = vec![];

        let mandatory_attribute_names = self
            .attribute_validators
            .iter()
            .filter(|(_, v)| {
                if let Some(conformance) = &v.conformance {
                    conformance.eq("M")
                } else {
                    false
                }
            })
            .map(|(attr_name, _)| attr_name)
            .collect::<Vec<&String>>();

        for (data_set_index, data_set) in self.data_sets.iter().enumerate() {
            for (record_index, record) in data_set
                .load(self.attribute_types.clone())?
                .iter()
                .enumerate()
            {
                let mut missing_attribute_names = mandatory_attribute_names.clone();
                for (k, v) in record.as_object().unwrap().iter() {
                    missing_attribute_names.retain(|n| n.ne(&k));

                    let attribute_validator = self.attribute_validators.get(k).ok_or_else(|| {
                        GenericError::from(format!(
                            "Data Set: {}, Record {}: '{}' attribute doesn't occur in OCA",
                            data_set_index, record_index, k
                        ))
                    });
                    match attribute_validator {
                        Ok(validator) => {
                            if let Err(errors) = self.validate_value(v, validator) {
                                for error in errors {
                                    validation_errors.push(GenericError::from(format!(
                                        "Data Set: {}, Record {}: {}",
                                        data_set_index, record_index, error
                                    )));
                                }
                            }
                        }
                        Err(err) => {
                            if let Some(config) = &self.constraints_config {
                                if config.fail_on_additional_attributes {
                                    validation_errors.push(err);
                                }
                            }
                            continue;
                        }
                    }
                }
                for missing_attribute_name in missing_attribute_names {
                    validation_errors.push(GenericError::from(format!(
                        "Data Set {}, Record {}: '{}' attribute is missing",
                        data_set_index, record_index, missing_attribute_name
                    )));
                }
            }
        }

        if validation_errors.is_empty() {
            Ok(())
        } else {
            Err(validation_errors)
        }
    }

    fn validate_value(
        &self,
        value: &Value,
        validator: &AttributeValidator,
    ) -> Result<(), Vec<String>> {
        let mut errors = vec![];

        if let Some(ref conformance) = validator.conformance {
            if conformance.eq("M") {
                match value {
                    Value::Null => {
                        errors.push(format!("'{}' value is mandatory", validator.attr_name));
                    }
                    Value::String(v) => {
                        if v.trim().is_empty() {
                            errors.push(format!("'{}' value is mandatory", validator.attr_name));
                        }
                    }
                    _ => {}
                }
            }
        }

        if !value.is_null() {
            match validator.attr_type {
                AttributeType::Text => {
                    if !value.is_string() {
                        errors.push(format!(
                            "'{}' value ({}) must be a Text type",
                            validator.attr_name, value
                        ));
                    } else if let Some(ref format) = validator.format {
                        let regex = Regex::new(format!("^{}$", format).as_str());
                        match regex {
                            Ok(re) => {
                                if !re.is_match(value.as_str().unwrap()) {
                                    errors.push(format!(
                                        "'{}' value ({}) must match defined format ({}) from Format overlay",
                                        validator.attr_name, value, format
                                    ));
                                }
                            }
                            Err(_) => {
                                errors.push(format!(
                                    "'{}' format definition is invalid",
                                    validator.attr_name
                                ));
                            }
                        }
                    }
                }
                AttributeType::Numeric => match value {
                    Value::Number(_) => {}
                    Value::String(v) => {
                        if let Err(_err) = v.parse::<f64>() {
                            errors.push(format!(
                                "'{}' value ({}) must be a Numeric type",
                                validator.attr_name, value
                            ));
                        }
                    }
                    _ => {
                        errors.push(format!(
                            "'{}' value ({}) must be a Numeric type",
                            validator.attr_name, value
                        ));
                    }
                },
                AttributeType::Boolean => {
                    if !value.is_boolean() {
                        errors.push(format!(
                            "'{}' value ({}) must be a Boolean type",
                            validator.attr_name, value
                        ));
                    }
                }
                AttributeType::Date => {
                    if !value.is_string() {
                        errors.push(format!(
                            "'{}' value ({}) must be a Date type",
                            validator.attr_name, value
                        ));
                    }
                }
                AttributeType::Binary => {
                    if !value.is_string() {
                        errors.push(format!(
                            "'{}' value ({}) must be a Binary type",
                            validator.attr_name, value
                        ));
                    }
                }
                AttributeType::Sai => {
                    if !value.is_object() {
                        errors.push(format!(
                            "'{}' value ({}) must be an object",
                            validator.attr_name, value
                        ));
                    }
                }
                AttributeType::ArrayText
                | AttributeType::ArrayNumeric
                | AttributeType::ArrayBoolean
                | AttributeType::ArrayDate
                | AttributeType::ArrayBinary
                | AttributeType::ArraySai => {
                    if !value.is_array() {
                        errors.push(format!(
                            "'{}' value ({}) must be an {}",
                            validator.attr_name,
                            value,
                            serde_json::to_string(&validator.attr_type).unwrap()
                        ));
                    } else {
                        let value_elements = value.as_array().unwrap();
                        if value_elements.is_empty() {
                            if let Some(conformance) = &validator.conformance {
                                if conformance.eq("M") {
                                    errors.push(format!(
                                        "'{}' value ({}) cannot be empty",
                                        validator.attr_name, value
                                    ));
                                }
                            }
                        }
                        for (i, element) in value_elements.iter().enumerate() {
                            if let Err(errs) =
                                self.validate_value(element, &validator.element(i).unwrap())
                            {
                                errors.extend(errs);
                            }
                        }
                    }
                }
            }

            if let Some(EntryCodes::Array(ref codes)) = validator.entry_codes {
                if value.is_string() && !codes.contains(&value.as_str().unwrap().to_string()) {
                    errors.push(format!(
                        "'{}' value ({}) must be one of {:?}",
                        validator.attr_name, value, codes
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn parse_oca_attributes_to_validators(oca: &OCA) -> HashMap<String, AttributeValidator> {
        let mut attribute_validators: HashMap<String, AttributeValidator> = HashMap::new();
        for (attr_name, attr_type) in &oca.capture_base.attributes {
            let mut validator = AttributeValidator::new(
                attr_name.to_string(),
                serde_json::from_str::<AttributeType>(format!("\"{}\"", attr_type).as_str())
                    .unwrap(),
            );

            for overlay in &oca.overlays {
                if overlay.attributes().contains(&attr_name) {
                    if overlay.overlay_type().contains("/character_encoding/") {
                        let ov = overlay
                            .as_any()
                            .downcast_ref::<overlay::CharacterEncoding>()
                            .unwrap();
                        validator.encoding = Some(
                            *ov.attr_character_encoding
                                .get(attr_name)
                                .unwrap_or(&ov.default_character_encoding),
                        )
                    } else if overlay.overlay_type().contains("/conformance/") {
                        let ov = overlay
                            .as_any()
                            .downcast_ref::<overlay::Conformance>()
                            .unwrap();
                        validator.conformance =
                            Some(ov.attr_conformance.get(attr_name).unwrap().to_string())
                    } else if overlay.overlay_type().contains("/format/") {
                        let ov = overlay.as_any().downcast_ref::<overlay::Format>().unwrap();
                        validator.format = Some(ov.attr_formats.get(attr_name).unwrap().to_string())
                    } else if overlay.overlay_type().contains("/entry_code/") {
                        let ov = overlay
                            .as_any()
                            .downcast_ref::<overlay::EntryCode>()
                            .unwrap();
                        validator.entry_codes =
                            Some(ov.attr_entry_codes.get(attr_name).unwrap().clone())
                    }
                }
            }
            attribute_validators.insert(attr_name.to_string(), validator);
        }

        attribute_validators
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_set::CSVDataSet;

    fn setup_oca() -> OCA {
        let common_assets_dir_path = format!("{}/../assets", env!("CARGO_MANIFEST_DIR"));
        let oca_result = oca_zip_resolver::resolve_from_zip(
            format!("{}/oca_bundle.zip", common_assets_dir_path).as_str(),
        );
        assert!(oca_result.is_ok());
        oca_result.unwrap()
    }

    #[test]
    fn validation_of_proper_data_set_should_return_successful_validation_result() {
        let oca = setup_oca();
        let mut validator = Validator::new(oca);
        validator.add_data_set(CSVDataSet::new(
            r#"email*;licenses*;number;numbers;date;dates;bool;bools
test@example.com;["A"];1;[22, "23"];01.01.1999;["01.01.2000"];true;[false, true]"#
                .to_string(),
        ));
        let validation_result = validator.validate();
        assert!(validation_result.is_ok());
    }

    #[test]
    fn validation_of_data_set_with_missing_attribute_should_return_failed_validation_result() {
        let oca = setup_oca();
        let mut validator = Validator::new(oca);
        validator.add_data_set(CSVDataSet::new(
            r#"email*
test@example.com"#
                .to_string(),
        ));
        let validation_result = validator.validate();
        assert!(validation_result.is_err());
    }

    #[test]
    fn validation_with_unfulfilled_constraints_should_return_failed_validation_result() {
        let oca = setup_oca();
        let mut validator = Validator::new(oca);
        validator.set_constraints(ConstraintsConfig {
            fail_on_additional_attributes: true,
        });
        validator.add_data_set(CSVDataSet::new(
            r#"email*;licenses*;additional
test@example.com;["A"];1"#
                .to_string(),
        ));
        let validation_result = validator.validate();

        assert!(validation_result.is_err());
    }

    #[test]
    fn validation_of_invalid_data_set_should_return_failed_validation_result() {
        let oca = setup_oca();
        let mut validator = Validator::new(oca);
        validator.add_data_set(CSVDataSet::new(
            r#"additional
1"#
            .to_string(),
        ));
        let validation_result = validator.validate();
        assert!(validation_result.is_err());
    }
}
