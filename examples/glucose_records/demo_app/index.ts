import { Transformer, resolveFromZip, CSVDataSet } from 'oca-data-transformer'
import * as fs from 'fs';

const data = fs.readFileSync('../data_set.csv', 'utf8');
const oca = resolveFromZip("../oca_bundle.zip")

const transformer = new Transformer(oca)

const mapping_overlay = `
  {
    "attribute_mapping": {
      "firstName": "first_name",
      "surname": "last_name"
    },
    "capture_base": "EzMTX96ZkwRBzkUBI9c8jiXfHa6CfKJyrd6uH6vjIUZc",
    "digest": "EBfxGsWV4aQGRfRsVH_tDfQCFaBnNqA7cq8YW1Q3DdwA",
    "type": "spec/overlays/mapping/1.0"
  }
`

const unit_overlay = `
  {
    "attribute_units": {
      "blood_glucose": "mmol/L"
    },
    "capture_base": "EzMTX96ZkwRBzkUBI9c8jiXfHa6CfKJyrd6uH6vjIUZc",
    "digest": "EwpEtyZgBF0B9KXEufQQdeS2aN1GL4JIMcpVRffH3_pg",
    "metric_system": "nonSI",
    "type": "spec/overlays/unit/1.0"
  }
`
transformer.addDataSet(
  new CSVDataSet(data.trimEnd(), ",")
).transform(
  [mapping_overlay, unit_overlay]
)

const result = transformer.getRawDatasets()
console.log(result)
