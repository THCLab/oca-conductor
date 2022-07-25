use oca_rust::state::{attribute::AttributeType, encoding::Encoding, entry_codes::EntryCodes};

pub struct AttributeValidator {
    pub attr_name: String,
    pub attr_type: AttributeType,
    pub entry_codes: Option<EntryCodes>,
    pub format: Option<String>,
    pub conformance: Option<String>,
    pub encoding: Option<Encoding>,
}

impl AttributeValidator {
    pub fn new(attr_name: String, attr_type: AttributeType) -> AttributeValidator {
        AttributeValidator {
            attr_name,
            attr_type,
            format: None,
            entry_codes: None,
            encoding: None,
            conformance: None,
        }
    }

    pub fn element(&self, index: usize) -> Option<AttributeValidator> {
        let attr_type = match self.attr_type {
            AttributeType::ArrayText => AttributeType::Text,
            AttributeType::ArrayNumeric => AttributeType::Numeric,
            AttributeType::ArrayBoolean => AttributeType::Boolean,
            AttributeType::ArrayDate => AttributeType::Date,
            AttributeType::ArrayBinary => AttributeType::Binary,
            AttributeType::ArraySai => AttributeType::Sai,
            _ => return None,
        };
        Some(AttributeValidator {
            attr_name: format!("{}[{}]", self.attr_name, index),
            attr_type,
            format: self.format.clone(),
            entry_codes: self.entry_codes.clone(),
            conformance: self.conformance.clone(),
            encoding: self.encoding,
        })
    }
}
