pub mod data_set_transformer;

use crate::data_set::DataSet;
use crate::errors::GenericError;
use crate::{validator::ConstraintsConfig, Validator};
use oca_rust::controller::load_oca;
use oca_rust::state::oca::{DynOverlay, OCA};
use said::derivation::SelfAddressing;

pub struct Transformer {
    oca: OCA,
    validator: Validator,
    data_sets: Vec<Box<dyn DataSet>>,
}

impl Transformer {
    pub fn new(oca: OCA) -> Self {
        let mut validator = Validator::new(
            load_oca(&mut serde_json::to_string(&oca).unwrap().as_bytes())
                .unwrap()
                .finalize(),
        );
        validator.set_constraints(ConstraintsConfig {
            fail_on_additional_attributes: true,
        });

        Self {
            oca,
            validator,
            data_sets: vec![],
        }
    }

    pub fn add_data_set(
        &mut self,
        data_set: Box<dyn DataSet>,
        overlays: Option<Vec<&str>>,
    ) -> Result<&mut Self, Vec<GenericError>> {
        let mut errors = vec![];
        let mut transformed_data_set = data_set.clone();

        if let Some(overlays) = overlays {
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
                                i
                            )))
                        }
                    }
                    Err(_) => errors.push(GenericError::from(format!(
                        "Overlay at position {}: Parsing failed. Invalid format.",
                        i
                    ))),
                }
            }

            if errors.is_empty() {
                let result =
                    data_set_transformer::transform_pre(&self.oca, additional_overlays, data_set);

                match result {
                    Ok(data_set) => transformed_data_set = data_set,
                    Err(errs) => errors.extend(errs.iter().map(|e| GenericError::from(e.clone()))),
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        self.validator.data_sets = vec![];
        self.validator.add_data_set(transformed_data_set.clone());
        match self.validator.validate() {
            Ok(_) => {
                self.data_sets.push(transformed_data_set);
                Ok(self)
            }
            Err(errors) => Err(errors),
        }
    }

    pub fn transform(&mut self, overlays: Vec<&str>) -> Result<&mut Self, Vec<GenericError>> {
        let mut errors = vec![];
        let mut target_overlays: Vec<DynOverlay> = vec![];

        let cb_json = serde_json::to_string(&self.oca.capture_base).unwrap();
        let oca_cb_sai = format!("{}", SelfAddressing::Blake3_256.derive(cb_json.as_bytes()));
        for (i, overlay_str) in overlays.iter().enumerate() {
            match serde_json::from_str::<DynOverlay>(overlay_str) {
                Ok(mut overlay) => {
                    if oca_cb_sai.eq(overlay.capture_base()) {
                        target_overlays.push(overlay);
                    } else {
                        errors.push(GenericError::from(format!(
                            "Overlay at position {}: Incompatible with OCA Capture Base.",
                            i
                        )))
                    }
                }
                Err(_) => errors.push(GenericError::from(format!(
                    "Overlay at position {}: Parsing failed. Invalid format.",
                    i
                ))),
            }
        }

        if self.data_sets.is_empty() {
            errors.push(GenericError::from("Dataset is empty"));
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        let mut transformed_data_sets = vec![];
        for (i, data_set) in self.data_sets.iter().enumerate() {
            let result =
                data_set_transformer::transform_post(&self.oca, &target_overlays, data_set.clone());

            match result {
                Ok(data_set) => transformed_data_sets.push(data_set),
                Err(errs) => errors.extend(
                    errs.iter()
                        .map(|e| GenericError::from(format!("DataSet {}: {}", i, e.clone()))),
                ),
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        self.data_sets = transformed_data_sets;

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
        let oca_result = oca_zip_resolver::resolve_from_zip(
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
            .add_data_set(
                CSVDataSet::new(
                    r#"e-mail*;licenses*
test@example.com;["A"]"#
                        .to_string(),
                ),
                Some(vec![
                    r#"
{
    "capture_base":"EKmZWuURpiUdl_YAMGQbLiossAntKt1DJ0gmUMYSz7Yo",
    "type":"spec/overlays/mapping/1.0",
    "attr_mapping": {
        "email*":"e-mail*"
    }
}
              "#,
                ]),
            )
            .unwrap()
            .add_data_set(
                CSVDataSet::new(
                    r#"e-mail;licenses*
test2@example.com;["B"]"#
                        .to_string(),
                ),
                Some(vec![
                    r#"
{
    "capture_base":"EKmZWuURpiUdl_YAMGQbLiossAntKt1DJ0gmUMYSz7Yo",
    "type":"spec/overlays/mapping/1.0",
    "attr_mapping": {
        "email*":"e-mail"
    }
}
              "#,
                ]),
            )
            .unwrap();

        assert_eq!(
            transformer.get_raw_datasets(),
            vec![
                "email*;licenses*\ntest@example.com;[\"A\"]",
                "email*;licenses*\ntest2@example.com;[\"B\"]"
            ]
        )
    }

    #[test]
    fn transform_with_attribute_mapping_overlay() {
        let oca = setup_oca();
        let mut transformer = Transformer::new(oca);
        transformer
            .add_data_set(
                CSVDataSet::new(
                    r#"e-mail*;licenses*
test@example.com;["A"]"#
                        .to_string(),
                ),
                Some(vec![
                    r#"
{
    "capture_base":"EKmZWuURpiUdl_YAMGQbLiossAntKt1DJ0gmUMYSz7Yo",
    "type":"spec/overlays/mapping/1.0",
    "attr_mapping": {
        "email*":"e-mail*"
    }
}
              "#,
                ]),
            )
            .unwrap()
            .add_data_set(
                CSVDataSet::new(
                    r#"e-mail;licenses*
test2@example.com;["B"]"#
                        .to_string(),
                ),
                Some(vec![
                    r#"
{
    "capture_base":"EKmZWuURpiUdl_YAMGQbLiossAntKt1DJ0gmUMYSz7Yo",
    "type":"spec/overlays/mapping/1.0",
    "attr_mapping": {
        "email*":"e-mail"
    }
}
              "#,
                ]),
            )
            .unwrap()
            .transform(vec![
                r#"
{
    "capture_base":"EKmZWuURpiUdl_YAMGQbLiossAntKt1DJ0gmUMYSz7Yo",
    "type":"spec/overlays/mapping/1.0",
    "attr_mapping": {
        "email*":"email:"
    }
}
              "#,
            ])
            .unwrap();

        assert_eq!(
            transformer.get_raw_datasets(),
            vec![
                "email:;licenses*\ntest@example.com;[\"A\"]",
                "email:;licenses*\ntest2@example.com;[\"B\"]"
            ]
        )
    }

    #[test]
    fn transform_data_with_entry_code_mapping_overlay() {
        let oca = setup_oca();
        let mut transformer = Transformer::new(oca);
        let result = transformer.add_data_set(
            CSVDataSet::new(
                r#"email*;licenses*
a@a.com;["a"]"#
                    .to_string(),
            ),
            Some(vec![
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
            ]),
        );

        assert!(result.is_ok());
        assert_eq!(
            transformer.get_raw_datasets(),
            vec!["email*;licenses*\na@a.com;[\"A\"]",]
        )
    }

    #[test]
    fn transform_with_entry_code_mapping_overlay() {
        let oca = setup_oca();
        let mut transformer = Transformer::new(oca);
        transformer
            .add_data_set(
                CSVDataSet::new(
                    r#"email*;licenses*
a@a.com;["A"]"#
                        .to_string(),
                ),
                None,
            )
            .unwrap()
            .transform(vec![
                r#"
{
    "capture_base":"EKmZWuURpiUdl_YAMGQbLiossAntKt1DJ0gmUMYSz7Yo",
    "type":"spec/overlays/entry_code_mapping/1.0",
    "attr_entry_codes_mapping": {
        "licenses*": [
            "A:1","B:2","C:3","D:4","E:5"
        ]
    }
}
              "#,
            ])
            .unwrap();

        assert_eq!(
            transformer.get_raw_datasets(),
            vec!["email*;licenses*\na@a.com;[\"1\"]",]
        )
    }

    #[test]
    fn transform_data_with_unit_overlay() {
        let oca = setup_oca();
        let mut transformer = Transformer::new(oca);
        let result = transformer.add_data_set(
            CSVDataSet::new(
                r#"email*;licenses*;number
a@a.com;["A"];3.2808"#
                    .to_string(),
            ),
            Some(vec![
                r#"
{
    "capture_base":"EKmZWuURpiUdl_YAMGQbLiossAntKt1DJ0gmUMYSz7Yo",
    "type":"spec/overlays/unit/1.0",
    "metric_system":"IU",
    "attr_units":{"number":"ft"}
}
              "#,
            ]),
        );

        assert!(result.is_ok());
        assert_eq!(
            transformer.get_raw_datasets(),
            vec!["email*;licenses*;number\na@a.com;[\"A\"];100.0",]
        )
    }

    #[test]
    fn transform_with_unit_overlay() {
        let oca = setup_oca();
        let mut transformer = Transformer::new(oca);
        transformer
            .add_data_set(
                CSVDataSet::new(
                    r#"email*;licenses*;number
a@a.com;["A"];100"#
                        .to_string(),
                ),
                None,
            )
            .unwrap()
            .transform(vec![
                r#"
{
    "capture_base":"EKmZWuURpiUdl_YAMGQbLiossAntKt1DJ0gmUMYSz7Yo",
    "type":"spec/overlays/unit/1.0",
    "metric_system":"SI",
    "attr_units":{"number":"m"}
}
              "#,
            ])
            .unwrap();

        assert_eq!(
            transformer.get_raw_datasets(),
            vec!["email*;licenses*;number\na@a.com;[\"A\"];1.0",]
        )
    }

    #[test]
    fn transform_data_with_invalid_overlay() {
        let oca = setup_oca();
        let mut transformer = Transformer::new(oca);
        let result = transformer.add_data_set(
            CSVDataSet::new(
                r#"e-mail*;licenses*
test@example.com;["A"]"#
                    .to_string(),
            ),
            Some(vec![r#"invalid"#]),
        );

        assert!(result.is_err())
    }
}
