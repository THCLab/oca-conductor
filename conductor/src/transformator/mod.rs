use crate::data_set::DataSet;
use oca_rust::state::oca::{overlay, DynOverlay, OCA};
use serde_json::Value;
use std::collections::BTreeMap;

pub fn transform_data(
    oca: &OCA,
    additional_overlays: Vec<DynOverlay>,
    data_sets: &[Box<dyn DataSet>],
) -> Result<Vec<Box<dyn DataSet>>, Vec<String>> {
    let mut attribute_mappings: BTreeMap<String, String> = BTreeMap::new();
    let mut entry_code_mappings: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut source_units: BTreeMap<String, String> = BTreeMap::new();
    let mut target_units: BTreeMap<String, String> = BTreeMap::new();
    let mut unit_transformation_operations: BTreeMap<String, Vec<Operation>> = BTreeMap::new();

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
    }
    for overlay in additional_overlays {
        if let Some(mappings) = get_attribute_mappings(&overlay) {
            attribute_mappings.extend(mappings);
        }
        if let Some(mappings) = get_entry_code_mappings(&overlay) {
            entry_code_mappings.extend(mappings);
        }
        if let Some(units) = get_units(&overlay) {
            source_units.extend(units);
        }
    }

    for (k, source_unit) in &source_units {
        if let Some(target_unit) = target_units.get(k) {
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
                            return Err(vec![r
                                .get("error")
                                .unwrap()
                                .as_str()
                                .unwrap()
                                .to_string()]);
                        }
                    } else {
                        return Err(vec![
                            "Error while transforming units. OCA Repository cannot return operations for unit transformation.".to_string()
                        ]);
                    }
                }
                Err(error) => return Err(vec![error.to_string()]),
            }

            unit_transformation_operations.insert(k.clone(), operations);
        }
    }

    let mut transformed_data_sets = vec![];
    for data_set in data_sets {
        let mut transformed_data_set = data_set.clone();
        if !attribute_mappings.is_empty() {
            transformed_data_set =
                transformed_data_set.transform_schema(attribute_mappings.clone());
        }

        if !unit_transformation_operations.is_empty() || !entry_code_mappings.is_empty() {
            transformed_data_set = transformed_data_set.transform_data(
                oca,
                entry_code_mappings.clone(),
                unit_transformation_operations.clone(),
            )
        }
        transformed_data_sets.push(transformed_data_set);
    }
    Ok(transformed_data_sets)
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

fn get_attribute_mappings(overlay: &DynOverlay) -> Option<BTreeMap<String, String>> {
    if overlay.overlay_type().contains("/mapping/") {
        let ov = overlay
            .as_any()
            .downcast_ref::<overlay::AttributeMapping>()
            .unwrap();
        return Some(
            ov.attr_mapping
                .values()
                .cloned()
                .zip(ov.attr_mapping.keys().cloned())
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

        for (attr_name, value) in &ov.attr_entry_codes_mapping {
            let mut mappings = BTreeMap::new();
            for v in value {
                let splitted = v.split(':').collect::<Vec<&str>>();
                mappings.insert(
                    splitted.get(0).unwrap().to_string(),
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
            ov.attr_units
                .keys()
                .cloned()
                .zip(
                    ov.attr_units
                        .values()
                        .map(|v| format!("{}:{}", ov.metric_system, v))
                        .collect::<Vec<String>>(),
                )
                .collect(),
        );
    }
    None
}
