import { Transformer, resolveFromZip, CSVDataSet } from 'oca-data-transformer'
import * as fs from 'fs';

const data = fs.readFileSync('../data_set.csv', 'utf8');
const oca = resolveFromZip("../oca_bundle.zip")

const transformer = new Transformer(oca)

const mapping_ov = `
{
        "capture_base":"Ev_RaB-gIOn8VAB3sg40mINxjiYRxdLVQrgce0aZbFcc",
        "type":"spec/overlays/mapping/1.0",
        "attr_mapping":{
            "first_name":"firstName",
            "last_name":"surname"
        }
    }
  `

const unit_overlay = `
{
  "capture_base":"Ev_RaB-gIOn8VAB3sg40mINxjiYRxdLVQrgce0aZbFcc",
  "type":"spec/overlays/unit/1.0",
  "metric_system":"SI",
  "attr_units":{"blood_glucose":"mg/dL"}
}
  `
transformer
  .addDataSet(new CSVDataSet(data.trimEnd()))
  .transformPre([mapping_ov, unit_overlay])

const result = transformer.getRawDatasets()
console.log(result)
