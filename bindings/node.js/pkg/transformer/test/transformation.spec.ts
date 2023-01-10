import { expect } from "chai"
import { resolveFromZip, Transformer, CSVDataSet } from ".."

describe("Transformer", () => {
  describe("#addDataSet()", () => {
    it("should return successful transformation result when transformation is valid", () => {
      const oca = resolveFromZip(`${__dirname}/../../../../../assets/oca_bundle.zip`)
      const transformer = new Transformer(oca)
      transformer.addDataSet(
        new CSVDataSet(
`e-mail*,licenses*
test@example.com,["a"]`, ','
        ), [
        `
{
  "attribute_mapping":{
    "email*":"e-mail*"
  },
  "capture_base":"Et7SxuRi_lK6blZmUO3X80Ji5lqMJe7DucrbUmhyzUzk",
  "digest":"Em51us0v3CuoYDZqxj4zB37w3lZHRjRyDa7TS9SJOJ7Q",
  "type":"spec/overlays/mapping/1.0"
}
        `,
        `
{
  "attribute_entry_codes_mapping":{
    "licenses*":["a:A", "b:B", "c:C", "d:D", "e:E"]
  },
  "capture_base":"Et7SxuRi_lK6blZmUO3X80Ji5lqMJe7DucrbUmhyzUzk",
  "digest":"EATJjWP-8p01ZHcQX2xB52VmaiMkiwVMG-VfBLyCs9So",
  "type":"spec/overlays/entry_code_mapping/1.0"
}
        `
      ])
      const result = transformer.getRawDatasets()
      expect(result.length).to.be.eq(1)
      expect(result[0]).to.be.eq('email*,licenses*\ntest@example.com,["A"]')
    })

    it("should throw errors when data_set is invalid", () => {
      const oca = resolveFromZip(`${__dirname}/../../../../../assets/oca_bundle.zip`)
      const transformer = new Transformer(oca)

      expect(
        () =>
          transformer.addDataSet(
            new CSVDataSet(
    `e-mail*
    test@example.com`, ','
            ), [
            `
{
  "attribute_mapping":{
    "email*":"e-mail*"
  },
  "capture_base":"Et7SxuRi_lK6blZmUO3X80Ji5lqMJe7DucrbUmhyzUzk",
  "digest":"Em51us0v3CuoYDZqxj4zB37w3lZHRjRyDa7TS9SJOJ7Q",
  "type":"spec/overlays/mapping/1.0"
}
            `
          ])
      ).to.throw()
    })
  })

  describe("#transform()", () => {
    it("should return successful transformation result when transformation is valid", () => {
      const oca = resolveFromZip(`${__dirname}/../../../../../assets/oca_bundle.zip`)
      const transformer = new Transformer(oca)
      transformer.addDataSet(
        new CSVDataSet(
`email*,licenses*
test@example.com,["A"]`, ','
        )
      ).transform([
        `
{
  "attribute_mapping":{
    "email*":"email:"
  },
  "capture_base":"Et7SxuRi_lK6blZmUO3X80Ji5lqMJe7DucrbUmhyzUzk",
  "digest":"EIVz6GUA-74ctvkF-cbuvw97RiHFL6YSs-oO1jsP1amo",
  "type":"spec/overlays/mapping/1.0"
}
        `,
        `
{
  "attribute_entry_codes_mapping":{
    "licenses*":["A:1", "B:2", "C:3", "D:4", "E:5"]
  },
  "capture_base":"Et7SxuRi_lK6blZmUO3X80Ji5lqMJe7DucrbUmhyzUzk",
  "digest":"ECSC1gNDlNjhrTgAVEdB2rZ3puJO-zAX5rv0w3wFOSX4",
  "type":"spec/overlays/entry_code_mapping/1.0"
}
        `
      ])
      const result = transformer.getRawDatasets()
      expect(result.length).to.be.eq(1)
      expect(result[0]).to.be.eq('email:,licenses*\ntest@example.com,["1"]')
    })
  })
})
