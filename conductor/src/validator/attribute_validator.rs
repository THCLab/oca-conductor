use oca_rs::state::{attribute::AttributeType, encoding::Encoding, entry_codes::EntryCodes};

pub struct AttributeValidator {
    pub attribute_name: String,
    pub attribute_type: AttributeType,
    pub entry_codes: Option<EntryCodes>,
    pub format: Option<String>,
    pub conformance: Option<String>,
    pub encoding: Option<Encoding>,
}

impl AttributeValidator {
    pub fn new(attribute_name: String, attribute_type: AttributeType) -> AttributeValidator {
        AttributeValidator {
            attribute_name,
            attribute_type,
            format: None,
            entry_codes: None,
            encoding: None,
            conformance: None,
        }
    }

    pub fn element(&self, index: usize) -> Option<AttributeValidator> {
        let attribute_type = match self.attribute_type {
            AttributeType::ArrayText => AttributeType::Text,
            AttributeType::ArrayNumeric => AttributeType::Numeric,
            AttributeType::ArrayBoolean => AttributeType::Boolean,
            AttributeType::ArrayDateTime => AttributeType::DateTime,
            AttributeType::ArrayBinary => AttributeType::Binary,
            AttributeType::ArrayReference => AttributeType::Reference,
            _ => return None,
        };
        Some(AttributeValidator {
            attribute_name: format!("{}[{}]", self.attribute_name, index),
            attribute_type,
            format: self.format.clone(),
            entry_codes: self.entry_codes.clone(),
            conformance: self.conformance.clone(),
            encoding: self.encoding,
        })
    }
}
