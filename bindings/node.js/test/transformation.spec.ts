import { expect } from "chai"
import { resolveFromZip, OcaConductor } from ".."

describe("OCAConductor", () => {
  describe("#transform_data()", () => {
    it("should return successful transformation result when transformation is valid", () => {
      const oca = resolveFromZip(`${__dirname}/../../../assets/oca_bundle.zip`)
      const conductor = OcaConductor.loadOca(oca)
      conductor.addDataSet([
          {
            "e-mail*": "test@example.com",
            "licenses*": ["a"]
          }
      ])
      const result = conductor.transformData([
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
    "attr_mapping": {
        "licenses*": [
            "a:A","b:B","c:C","d:D","e:E"
        ]
    }
}
        `
      ])
      expect(result.success).to.be.true
      expect(result.result[0]).to.not.have.property('e-mail*')
      expect(result.result[0]).to.have.property('email*')
      expect(result.result[0]).to.not.nested.include({ 'licenses*[0]': 'a' })
      expect(result.result[0]).to.nested.include({ 'licenses*[0]': 'A' })
    })
  })
})
