import { expect } from "chai"
import { resolveFromZip, OcaConductor } from ".."

describe("OCAConductor", () => {
  describe("#loadOca()", () => {
    it("should throw error when OCA object is invalid", () => {
      const oca = {}
      expect(
        () => OcaConductor.loadOca(oca)
      ).to.throw()
    })
  })

  describe("#validate()", () => {
    it("should return successful validation result when data set is valid", () => {
      const oca = resolveFromZip(`${__dirname}/../../../assets/oca_bundle.zip`)
      const conductor = OcaConductor.loadOca(oca)
      conductor.addDataSet([
          {
            "email*": "test@example.com",
            "licenses*": ["A"],
            "number": 24,
            "numbers": [22, "23"],
            "date": "01.01.1999",
            "dates": ["01.01.2000"],
            "bool": true,
            "bools": [false, true]
          }
      ])
      const result = conductor.validate()
      expect(result.success).to.be.true
    })
  })
})
