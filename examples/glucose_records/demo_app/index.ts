import { Transformer, resolveFromZip, CSVDataSet } from 'oca-data-transformer'
import * as fs from 'fs';

const data = fs.readFileSync('../data_set.csv', 'utf8');
const oca = resolveFromZip("../oca_bundle.zip")

const transformer = new Transformer(oca)

const unit_overlay = `
  {
    "attribute_units": {
      "valueQuantity_14745-4": "mg/dL"
    },
    "capture_base": "E9V_1S5JwM9HA5qut8xfKlZLKuZTCJlEzQFNJs4MbBuY",
    "digest": "E1-FcPs6Y_ct6UTVoE3Ok0juGLFmOic3uUywRUJBn-P8",
    "metric_system": "SI",
    "type": "spec/overlays/unit/1.0"
  }
`
transformer.addDataSet(
  new CSVDataSet(data.trimEnd(), ",")
).transform(
  [unit_overlay]
)

const result = transformer.getRawDatasets()
console.log(result)
