use oca_rust::state::oca::{overlay, DynOverlay};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

pub fn transform_data(
    oca_overlays: &[DynOverlay],
    additional_overlays: Vec<DynOverlay>,
    data_sets: Vec<Value>,
) -> Result<Vec<Value>, Vec<String>> {
    let mut transformed_data_set = vec![];

    let mut attribute_mappings: BTreeMap<String, String> = BTreeMap::new();
    let mut entry_code_mappings: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut source_units: BTreeMap<String, String> = BTreeMap::new();
    let mut target_units: BTreeMap<String, String> = BTreeMap::new();
    let mut unit_transformation_operations: BTreeMap<String, Vec<Operation>> = BTreeMap::new();

    for overlay in oca_overlays {
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
                            let ops = r.get("result").unwrap()
                                .get(format!("{}->{}", source_unit, target_unit)).unwrap()
                                .as_array().unwrap();
                            for op in ops {
                                let o = op.as_object().unwrap();
                                if let Value::String(op_sign) = o.get("op").unwrap() {
                                    if op_sign.eq("*") {
                                        operations.push(
                                            Operation::new(OpType::Multiply, o.get("value").unwrap().as_f64().unwrap())
                                        );
                                    } else if op_sign.eq("/") {
                                        operations.push(
                                            Operation::new(OpType::Divide, o.get("value").unwrap().as_f64().unwrap())
                                        );
                                    } else if op_sign.eq("+") {
                                        operations.push(
                                            Operation::new(OpType::Add, o.get("value").unwrap().as_f64().unwrap())
                                        );
                                    } else if op_sign.eq("-") {
                                        operations.push(
                                            Operation::new(OpType::Subtract, o.get("value").unwrap().as_f64().unwrap())
                                        );
                                    }
                                }
                            }
                        } else {
                            return Err(vec![
                                r.get("error").unwrap().as_str().unwrap().to_string()
                            ])
                        }
                    } else {
                        return Err(vec![
                            "Error while transforming units. OCA Repository cannot return operations for unit transformation.".to_string()
                        ])
                    }
                }
                Err(error) => {
                    return Err(vec![error.to_string()])
                }
            }

            unit_transformation_operations.insert(format!("{}->{}", source_unit, target_unit), operations);
        }
    }

    for data_set in data_sets {
        let data_set_map = data_set.as_object().unwrap();
        let mut transformed_data = Map::new();
        for (k, v) in data_set_map {
            let mut key = k.to_string();
            let mut value = v.clone();
            if let Some(mapped_key) = attribute_mappings.get(k) {
                key = mapped_key.to_string();
            }
            if let Some(mapped_entries) = entry_code_mappings.get(k) {
                match value {
                    Value::Array(ref values_vec) => {
                        let mut mapped_values = vec![];
                        for v in values_vec {
                            match mapped_entries.get(v.as_str().unwrap()) {
                                Some(mapped_entry) => {
                                    mapped_values.push(Value::String(mapped_entry.to_string()));
                                }
                                None => {
                                    mapped_values.push(v.clone());
                                }
                            }
                        }
                        value = Value::Array(mapped_values);
                    }
                    Value::String(_) => {
                        if let Some(mapped_entry) = mapped_entries.get(value.as_str().unwrap()) {
                            value = Value::String(mapped_entry.to_string());
                        };
                    }
                    _ => ()
                }
            }
            if let Some(source_unit) = source_units.get(k) {
                if let Value::Number(num) = &value {
                    if let Some(target_unit) = target_units.get(k) {
                        value = Value::Number(
                            serde_json::value::Number::from_f64(
                                calculate_value_units(
                                    num.as_f64().unwrap(),
                                    unit_transformation_operations.get(&format!("{}->{}", source_unit, target_unit)).unwrap()
                                )
                            ).unwrap()
                        );
                    }
                }
            }
            transformed_data.insert(key, value);
        }

        transformed_data_set.push(Value::Object(transformed_data));
    }
    Ok(transformed_data_set)
}

fn calculate_value_units(value: f64, operations: &[Operation]) -> f64 {
    let mut result = value;
    for operation in operations {
        result = apply_operation(result, operation);
    }

    result
}

fn apply_operation(value: f64, operation: &Operation) -> f64 {
    match operation.op {
        OpType::Multiply => value * operation.value,
        OpType::Divide => value / operation.value,
        OpType::Add => value + operation.value,
        OpType::Subtract => value - operation.value,
    }
}

enum OpType {
    Multiply,
    Divide,
    Add,
    Subtract
}

struct Operation {
    op: OpType,
    value: f64
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
            ov.attr_mappings
                .values()
                .cloned()
                .zip(ov.attr_mappings.keys().cloned())
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

        for (attr_name, value) in &ov.attr_entry_codes_mappings {
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
        let ov = overlay
            .as_any()
            .downcast_ref::<overlay::Unit>()
            .unwrap();
        return Some(
            ov.attr_units
                .keys()
                .cloned()
                .zip(ov.attr_units.values().map(|v| format!("{}:{}", ov.metric_system, v)).collect::<Vec<String>>())
                .collect(),
        );
    }
    None
}
