use crate::ConstraintsConfig;
use oca_rust::state::{attribute::AttributeType, entry_codes::EntryCodes, oca::overlay, oca::OCA};
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;

mod attribute_validator;
use attribute_validator::AttributeValidator;

pub struct OCAValidator {
    constraints_config: Option<ConstraintsConfig>,
    attribute_validators: HashMap<String, AttributeValidator>,
}

impl OCAValidator {
    pub fn new(oca: &OCA) -> OCAValidator {
        OCAValidator {
            constraints_config: None,
            attribute_validators: OCAValidator::parse_oca_attributes_to_validators(oca),
        }
    }

    pub fn set_constraints(&mut self, config: ConstraintsConfig) {
        self.constraints_config = Some(config);
    }

    pub fn validate(&self, data_sets: Vec<Value>) -> Result<(), Vec<String>> {
        let mut validation_errors: Vec<String> = vec![];

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

        for (record_index, record) in data_sets.iter().enumerate() {
            let mut missing_attribute_names = mandatory_attribute_names.clone();
            for (k, v) in record.as_object().unwrap().iter() {
                missing_attribute_names.retain(|n| n.ne(&k));

                let attribute_validator = self.attribute_validators.get(k).ok_or_else(|| {
                    format!(
                        "Record {}: '{}' attribute doesn't occur in OCA",
                        record_index, k
                    )
                });
                match attribute_validator {
                    Ok(validator) => {
                        if let Err(errors) = self.validate_value(v, validator) {
                            for error in errors {
                                validation_errors
                                    .push(format!("Record {}: {}", record_index, error));
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
                validation_errors.push(format!(
                    "Record {}: '{}' attribute is missing",
                    record_index, missing_attribute_name
                ));
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
                            },
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
                                        validator.attr_name,
                                        value
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
