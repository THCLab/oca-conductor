import { expect } from "chai"
import { resolveFromZip, Validator, CSVDataSet } from ".."

describe("Validator", () => {
  describe("constructor", () => {
    it("should throw error when OCA object is invalid", () => {
      const oca = {}
      expect(
        () => new Validator(oca)
      ).to.throw()
    })
  })

  describe("#validate()", () => {
    it("should return successful validation result when data set is valid", () => {
      const oca = resolveFromZip(`${__dirname}/../../../../../assets/oca_bundle.zip`)
      const validator = new Validator(oca)
      validator.addDataSet(
        new CSVDataSet(
`email*;licenses*;number;numbers;date;dates;bool;bools
test@example.com;["A"];24;[22, "23"];01.01.1999;["01.01.2000"];true;[false, true]`
        )
      )
      const result = validator.validate()
      expect(result.success).to.be.true
    })
  })
})
