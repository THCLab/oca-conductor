pub mod data_set_transformer;

use crate::data_set::DataSet;
use crate::errors::GenericError;
use oca_rust::state::oca::{DynOverlay, OCA};
use said::derivation::SelfAddressing;

pub struct Transformer {
    oca: OCA,
    data_sets: Vec<Box<dyn DataSet>>,
}

impl Transformer {
    pub fn new(oca: OCA) -> Self {
        Self {
            oca,
            data_sets: vec![],
        }
    }

    pub fn add_data_set<T: 'static>(&mut self, data_set: T) -> &mut Self
    where
        T: DataSet,
    {
        self.data_sets.push(Box::new(data_set));
        self
    }

    pub fn transform_pre(&mut self, overlays: Vec<&str>) -> Result<&mut Self, Vec<GenericError>> {
        let mut errors = vec![];
        let mut additional_overlays: Vec<DynOverlay> = vec![];

        let cb_json = serde_json::to_string(&self.oca.capture_base).unwrap();
        let oca_cb_sai = format!("{}", SelfAddressing::Blake3_256.derive(cb_json.as_bytes()));
        for (i, overlay_str) in overlays.iter().enumerate() {
            match serde_json::from_str::<DynOverlay>(overlay_str) {
                Ok(mut overlay) => {
                    if oca_cb_sai.eq(overlay.capture_base()) {
                        additional_overlays.push(overlay);
                    } else {
                        errors.push(GenericError::from(format!(
                            "Overlay at position {}: Incompatible with OCA Capture Base.",
                            i + 1
                        )))
                    }
                }
                Err(_) => errors.push(GenericError::from(format!(
                    "Overlay at position {}: Parsing failed. Invalid format.",
                    i + 1
                ))),
            }
        }

        let last_data_set_result = self.data_sets.pop().ok_or_else(|| {
            errors.push(GenericError::from("Dataset is empty"));
        });

        if !errors.is_empty() {
            return Err(errors);
        }

        let result = data_set_transformer::transform_data(
            &self.oca,
            additional_overlays,
            last_data_set_result.unwrap(),
        );

        match result {
            Ok(data_set) => self.data_sets.push(data_set),
            Err(errs) => errors.extend(errs.iter().map(|e| GenericError::from(e.clone()))),
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(self)
    }

    pub fn get_raw_datasets(&self) -> Vec<String> {
        self.data_sets.iter().map(|ds| ds.get_raw()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_set::CSVDataSet;

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
        let mut transformer = Transformer::new(oca);
        transformer
            .add_data_set(CSVDataSet::new(
                r#"e-mail*
test@example.com"#
                    .to_string(),
            ))
            .transform_pre(vec![
                r#"
{
    "capture_base":"EKmZWuURpiUdl_YAMGQbLiossAntKt1DJ0gmUMYSz7Yo",
    "type":"spec/overlays/mapping/1.0",
    "attr_mapping": {
        "email*":"e-mail*"
    }
}
              "#,
            ])
            .unwrap()
            .add_data_set(CSVDataSet::new(
                r#"e-mail
test2@example.com"#
                    .to_string(),
            ))
            .transform_pre(vec![
                r#"
{
    "capture_base":"EKmZWuURpiUdl_YAMGQbLiossAntKt1DJ0gmUMYSz7Yo",
    "type":"spec/overlays/mapping/1.0",
    "attr_mapping": {
        "email*":"e-mail"
    }
}
              "#,
            ])
            .unwrap();

        assert_eq!(
            transformer.get_raw_datasets(),
            vec!["email*\ntest@example.com", "email*\ntest2@example.com"]
        )
    }

    #[test]
    fn transform_data_with_entry_code_mapping_overlay() {
        let oca = setup_oca();
        let mut transformer = Transformer::new(oca);
        transformer
            .add_data_set(CSVDataSet::new(
                r#"licenses*
["a"]"#
                    .to_string(),
            ))
            .transform_pre(vec![
                r#"
{
    "capture_base":"EKmZWuURpiUdl_YAMGQbLiossAntKt1DJ0gmUMYSz7Yo",
    "type":"spec/overlays/entry_code_mapping/1.0",
    "attr_entry_codes_mapping": {
        "licenses*": [
            "a:A","b:B","c:C","d:D","e:E"
        ]
    }
}
              "#,
            ]);

        assert_eq!(transformer.get_raw_datasets(), vec!["licenses*\n[\"A\"]",])
    }

    #[test]
    fn transform_data_with_unit_overlay() {
        let oca = setup_oca();
        let mut transformer = Transformer::new(oca);
        transformer
            .add_data_set(CSVDataSet::new(
                r#"number
3.2808"#
                    .to_string(),
            ))
            .transform_pre(vec![
                r#"
{
    "capture_base":"EKmZWuURpiUdl_YAMGQbLiossAntKt1DJ0gmUMYSz7Yo",
    "type":"spec/overlays/unit/1.0",
    "metric_system":"IU",
    "attr_units":{"number":"ft"}
}
              "#,
            ]);

        assert_eq!(transformer.get_raw_datasets(), vec!["number\n100.0",])
    }

    #[test]
    fn transform_data_with_invalid_overlay() {
        let oca = setup_oca();
        let mut transformer = Transformer::new(oca);
        let result = transformer
            .add_data_set(CSVDataSet::new(
                r#"e-mail*
test@example.com"#
                    .to_string(),
            ))
            .transform_pre(vec![r#"invalid"#]);

        assert!(result.is_err())
    }
}
