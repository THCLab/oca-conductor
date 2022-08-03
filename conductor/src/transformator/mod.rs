use oca_rust::state::oca::{overlay, DynOverlay};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

pub fn transform_data(
    oca_overlays: &[DynOverlay],
    additional_overlays: Vec<DynOverlay>,
    data_sets: Vec<Value>,
) -> Vec<Value> {
    let mut transformed_data_set = vec![];

    let mut attribute_mappings: BTreeMap<String, String> = BTreeMap::new();
    let mut entry_code_mappings: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();

    for overlay in oca_overlays {
        if let Some(mappings) = get_attribute_mappings(overlay) {
            attribute_mappings.extend(mappings);
        }
        if let Some(mappings) = get_entry_code_mappings(overlay) {
            entry_code_mappings.extend(mappings);
        }
    }
    for overlay in additional_overlays {
        if let Some(mappings) = get_attribute_mappings(&overlay) {
            attribute_mappings.extend(mappings);
        }
        if let Some(mappings) = get_entry_code_mappings(&overlay) {
            entry_code_mappings.extend(mappings);
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
                    _ => {}
                }
            }
            transformed_data.insert(key, value);
        }

        transformed_data_set.push(Value::Object(transformed_data));
    }
    transformed_data_set
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

        for (attr_name, value) in &ov.attr_mapping {
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
