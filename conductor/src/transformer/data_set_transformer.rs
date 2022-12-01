use crate::data_set::DataSet;
use oca_rust::state::oca::{overlay, DynOverlay, OCA};
use serde_json::Value;
use std::collections::BTreeMap;

pub fn transform_pre(
    oca: &OCA,
    additional_overlays: Vec<DynOverlay>,
    data_set: Box<dyn DataSet>,
) -> Result<Box<dyn DataSet>, Vec<String>> {
    let mut attribute_mappings: BTreeMap<String, String> = BTreeMap::new();
    let mut entry_code_mappings: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut target_units: BTreeMap<String, String> = BTreeMap::new();
    let mut unit_transformation_operations: BTreeMap<String, Vec<Operation>> = BTreeMap::new();
    let mut subset_attributes: Vec<String> = vec![];

    let mut transformed_data_set = data_set.clone();

    for overlay in &oca.overlays {
        if let Some(mappings) = get_attribute_mappings(overlay) {
            attribute_mappings.extend(mappings);
        }
        if let Some(mappings) = get_entry_code_mappings(overlay) {
            entry_code_mappings.extend(mappings);
        }
        if let Some(units) = get_units(overlay) {
            target_units.extend(units);
        }
        if let Some(attributes) = get_subset_attributes(overlay) {
            subset_attributes.extend(attributes);
        }
    }

    if !unit_transformation_operations.is_empty() || !entry_code_mappings.is_empty() {
        transformed_data_set = transformed_data_set
            .transform_data(
                oca,
                entry_code_mappings.clone(),
                unit_transformation_operations.clone(),
            )
            .unwrap()
    }

    if !attribute_mappings.is_empty() || !subset_attributes.is_empty() {
        let mut subset_attributes_op = None;
        if !subset_attributes.is_empty() {
            subset_attributes_op = Some(subset_attributes.clone());
        }
        transformed_data_set = transformed_data_set
            .transform_schema(attribute_mappings.clone(), subset_attributes_op)
            .unwrap();
    }

    for overlay in additional_overlays {
        attribute_mappings = BTreeMap::new();
        entry_code_mappings = BTreeMap::new();
        subset_attributes = vec![];
        let mut source_units: BTreeMap<String, String> = BTreeMap::new();
        if let Some(mappings) = get_attribute_mappings(&overlay) {
            attribute_mappings.extend(mappings);
        }
        if let Some(mappings) = get_entry_code_mappings(&overlay) {
            entry_code_mappings.extend(mappings);
        }
        if let Some(units) = get_units(&overlay) {
            source_units.extend(units);
        }
        if let Some(attributes) = get_subset_attributes(&overlay) {
            subset_attributes.extend(attributes);
        }

        for (k, source_unit) in &source_units {
            if let Some(target_unit) = target_units.get(k) {
                if unit_transformation_operations.get(k).is_none() {
                    match get_operations(source_unit.clone(), target_unit.clone()) {
                        Ok(operations) => {
                            unit_transformation_operations.insert(k.clone(), operations);
                        }
                        Err(errors) => return Err(errors),
                    }
                }
            }
        }
    }

    if !unit_transformation_operations.is_empty() || !entry_code_mappings.is_empty() {
        transformed_data_set = transformed_data_set
            .transform_data(
                oca,
                entry_code_mappings.clone(),
                unit_transformation_operations.clone(),
            )
            .unwrap()
    }

    if !attribute_mappings.is_empty() || !subset_attributes.is_empty() {
        let mut subset_attributes_op = None;
        if !subset_attributes.is_empty() {
            subset_attributes_op = Some(subset_attributes.clone());
        }
        transformed_data_set = transformed_data_set
            .transform_schema(attribute_mappings.clone(), subset_attributes_op)
            .unwrap();
    }
    Ok(transformed_data_set)
}

pub fn transform_post(
    oca: &OCA,
    target_overlays: &Vec<DynOverlay>,
    data_set: Box<dyn DataSet>,
) -> Result<Box<dyn DataSet>, Vec<String>> {
    let mut attribute_mappings: BTreeMap<String, String> = BTreeMap::new();
    let mut entry_code_mappings: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut source_units: BTreeMap<String, String> = BTreeMap::new();
    let mut target_units: BTreeMap<String, String> = BTreeMap::new();
    let mut unit_transformation_operations: BTreeMap<String, Vec<Operation>> = BTreeMap::new();
    let mut subset_attributes: Vec<String> = vec![];

    let mut transformed_data_set = data_set.clone();

    for overlay in &oca.overlays {
        if let Some(mappings) = get_attribute_mappings(overlay) {
            let mut swapped_mappings = BTreeMap::new();
            for (k, v) in &mappings {
                swapped_mappings.insert(v.clone(), k.clone());
            }
            attribute_mappings.extend(swapped_mappings);
        }
        if let Some(mappings) = get_entry_code_mappings(overlay) {
            entry_code_mappings.extend(mappings);
        }
        if let Some(units) = get_units(overlay) {
            source_units.extend(units);
        }
    }
    for overlay in target_overlays {
        if let Some(mappings) = get_attribute_mappings(overlay) {
            let mut swapped_mappings = BTreeMap::new();
            for (k, v) in mappings {
                swapped_mappings.insert(v, k);
            }
            attribute_mappings.extend(swapped_mappings);
        }
        if let Some(mappings) = get_entry_code_mappings(overlay) {
            entry_code_mappings.extend(mappings);
        }
        if let Some(units) = get_units(overlay) {
            target_units.extend(units);
        }
        if let Some(attributes) = get_subset_attributes(overlay) {
            subset_attributes.extend(attributes);
        }

        for (k, target_unit) in &target_units {
            if let Some(source_unit) = source_units.get(k) {
                if unit_transformation_operations.get(k).is_none() {
                    match get_operations(source_unit.clone(), target_unit.clone()) {
                        Ok(operations) => {
                            unit_transformation_operations.insert(k.clone(), operations);
                        }
                        Err(errors) => return Err(errors),
                    }
                }
            }
        }
    }

    if !attribute_mappings.is_empty() || !subset_attributes.is_empty() {
        let mut subset_attributes_op = None;
        if !subset_attributes.is_empty() {
            subset_attributes_op = Some(subset_attributes.clone());
        }
        transformed_data_set = transformed_data_set
            .transform_schema(attribute_mappings.clone(), subset_attributes_op)
            .unwrap();
    }

    if !unit_transformation_operations.is_empty() || !entry_code_mappings.is_empty() {
        transformed_data_set = transformed_data_set
            .transform_data(
                oca,
                entry_code_mappings.clone(),
                unit_transformation_operations.clone(),
            )
            .unwrap()
    }
    Ok(transformed_data_set)
}

#[derive(Clone)]
pub enum OpType {
    Multiply,
    Divide,
    Add,
    Subtract,
}

#[derive(Clone)]
pub struct Operation {
    pub op: OpType,
    pub value: f64,
}

impl Operation {
    pub fn new(op: OpType, value: f64) -> Self {
        Self { op, value }
    }
}

fn get_operations(source_unit: String, target_unit: String) -> Result<Vec<Operation>, Vec<String>> {
    let mut operations = vec![];

    let request_url = format!("https://repository.oca.argo.colossi.network/api/v0.1/transformations/units?source={}&target={}",
            source_unit, target_unit);
    let response = reqwest::blocking::get(&request_url);
    match response {
        Ok(res) => {
            let result = res.json::<Value>();
            if let Ok(Value::Object(r)) = result {
                if r.get("success").unwrap().as_bool().unwrap() {
                    let ops = r
                        .get("result")
                        .unwrap()
                        .get(format!("{}->{}", source_unit, target_unit))
                        .unwrap()
                        .as_array()
                        .unwrap();
                    for op in ops {
                        let o = op.as_object().unwrap();
                        if let Value::String(op_sign) = o.get("op").unwrap() {
                            if op_sign.eq("*") {
                                operations.push(Operation::new(
                                    OpType::Multiply,
                                    o.get("value").unwrap().as_f64().unwrap(),
                                ));
                            } else if op_sign.eq("/") {
                                operations.push(Operation::new(
                                    OpType::Divide,
                                    o.get("value").unwrap().as_f64().unwrap(),
                                ));
                            } else if op_sign.eq("+") {
                                operations.push(Operation::new(
                                    OpType::Add,
                                    o.get("value").unwrap().as_f64().unwrap(),
                                ));
                            } else if op_sign.eq("-") {
                                operations.push(Operation::new(
                                    OpType::Subtract,
                                    o.get("value").unwrap().as_f64().unwrap(),
                                ));
                            }
                        }
                    }
                } else {
                    return Err(vec![r.get("error").unwrap().as_str().unwrap().to_string()]);
                }
            } else {
                return Err(vec![
                    "Error while transforming units. OCA Repository cannot return operations for unit transformation.".to_string()
                ]);
            }
        }
        Err(error) => return Err(vec![error.to_string()]),
    }

    Ok(operations)
}

fn get_attribute_mappings(overlay: &DynOverlay) -> Option<BTreeMap<String, String>> {
    if overlay.overlay_type().contains("/mapping/") {
        let ov = overlay
            .as_any()
            .downcast_ref::<overlay::AttributeMapping>()
            .unwrap();
        return Some(
            ov.attribute_mapping
                .values()
                .cloned()
                .zip(ov.attribute_mapping.keys().cloned())
                .collect(),
        );
    }
    None
}

fn get_entry_code_mappings(
    overlay: &DynOverlay,
) -> Option<BTreeMap<String, BTreeMap<String, String>>> {
    if overlay.overlay_type().contains("/entry_code_mapping/") {
        let ov = overlay
            .as_any()
            .downcast_ref::<overlay::EntryCodeMapping>()
            .unwrap();
        let mut entry_code_mappings_tmp = BTreeMap::new();

        for (attr_name, value) in &ov.attribute_entry_codes_mapping {
            let mut mappings = BTreeMap::new();
            for v in value {
                let splitted = v.split(':').collect::<Vec<&str>>();
                mappings.insert(
                    splitted.first().unwrap().to_string(),
                    splitted.get(1).unwrap().to_string(),
                );
            }
            entry_code_mappings_tmp.insert(attr_name.clone(), mappings);
        }
        return Some(entry_code_mappings_tmp);
    }
    None
}

fn get_units(overlay: &DynOverlay) -> Option<BTreeMap<String, String>> {
    if overlay.overlay_type().contains("/unit/") {
        let ov = overlay.as_any().downcast_ref::<overlay::Unit>().unwrap();
        return Some(
            ov.attribute_units
                .keys()
                .cloned()
                .zip(
                    ov.attribute_units
                        .values()
                        .map(|v| format!("{}:{}", ov.metric_system, v))
                        .collect::<Vec<String>>(),
                )
                .collect(),
        );
    }
    None
}

fn get_subset_attributes(overlay: &DynOverlay) -> Option<Vec<String>> {
    if overlay.overlay_type().contains("/subset/") {
        if let Some(ov) = overlay.as_any().downcast_ref::<overlay::Subset>() {
            return Some(ov.attributes.clone());
        }
    }
    None
}
