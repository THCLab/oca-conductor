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

      const result = validator.validate({
        'email*': 'test@example.com',
        'licenses*': ["A"],
        number: 24,
        numbers: [22, "23"],
        date: '01.01.1999',
        dates: ["01.01.2000"],
        bool: true,
        bools: [false, true]
      })

      expect(result.success).to.be.true
    })

    it("should return unsuccessful validation result when record is invalid", () => {
      const oca = resolveFromZip(`${__dirname}/../../../../../assets/oca_bundle.zip`)
      const validator = new Validator(oca)

      const result = validator.validate({
        'email*': 'invalid@email',
        number: 'text',
        bools: [false, "text"]
      })

      expect(result.success).to.be.false
    })

    it("should return unsuccessful validation result when data set has invalid records", () => {
      const oca = resolveFromZip(`${__dirname}/../../../../../assets/oca_bundle.zip`)
      const validator = new Validator(oca)

      const result = validator.validate([{
        'email*': 'invalid@email',
        number: 'text',
        bools: [false, "text"]
      }, {
        'email*': 'test@example.com',
        'licenses*': ["A"],
      }, {
        invalid: 'record'
      }])

      expect(result.success).to.be.false
    })

    it("should return successful validation result when data set has addtional attribute", () => {
      const oca = resolveFromZip(`${__dirname}/../../../../../assets/oca_bundle.zip`)
      const validator = new Validator(oca)
      validator.setConstraints({
        failOnAdditionalAttributes: true,
      })

      const result = validator.validate({
        'email*': 'test@example.com',
        'licenses*': ["A"],
        number: 24,
        numbers: [22, "23"],
        date: '01.01.1999',
        dates: ["01.01.2000"],
        bool: true,
        bools: [false, true],
        additional: 'attribute'
      })

      expect(result.success).to.be.false
    })
  })
})
