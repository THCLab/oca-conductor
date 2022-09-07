import { expect } from "chai"
import { resolveFromZip, Transformer, CSVDataSet } from ".."

describe("Transformer", () => {
  describe("#transform_pre()", () => {
    it("should return successful transformation result when transformation is valid", () => {
      const oca = resolveFromZip(`${__dirname}/../../../../../assets/oca_bundle.zip`)
      const transformer = new Transformer(oca)
      transformer.addDataSet(
        new CSVDataSet(
`e-mail*;licenses*
test@example.com;["a"]`
        )
      ).transformPre([
        `
{
    "capture_base":"EKmZWuURpiUdl_YAMGQbLiossAntKt1DJ0gmUMYSz7Yo",
    "type":"spec/overlays/mapping/1.0",
    "attr_mapping": {
        "email*":"e-mail*"
    }
}
        `,
        `
{
    "capture_base":"EKmZWuURpiUdl_YAMGQbLiossAntKt1DJ0gmUMYSz7Yo",
    "type":"spec/overlays/entry_code_mapping/1.0",
    "attr_entry_codes_mapping": {
        "licenses*": [
            "a:A","b:B","c:C","d:D","e:E"
        ]
    }
}
        `
      ])
      const result = transformer.getRawDatasets()
      expect(result.length).to.be.eq(1)
      expect(result[0]).to.be.eq('email*;licenses*\n"test@example.com";["A"]')
    })
  })
})
