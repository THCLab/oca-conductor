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
    "capture_base":"EGKvvpidW_ytpxsdiPedznDTHAgoRL2iWBy0d2pfCSW8",
    "type":"spec/overlays/mapping/1.0",
    "attribute_mapping": {
        "email*":"e-mail*"
    }
}
        `,
        `
{
    "capture_base":"EGKvvpidW_ytpxsdiPedznDTHAgoRL2iWBy0d2pfCSW8",
    "type":"spec/overlays/entry_code_mapping/1.0",
    "attribute_entry_codes_mapping": {
        "licenses*": [
            "a:A","b:B","c:C","d:D","e:E"
        ]
    }
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
        "capture_base":"EGKvvpidW_ytpxsdiPedznDTHAgoRL2iWBy0d2pfCSW8",
        "type":"spec/overlays/mapping/1.0",
        "attribute_mapping": {
            "email*":"e-mail*"
        }
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
    "capture_base":"EGKvvpidW_ytpxsdiPedznDTHAgoRL2iWBy0d2pfCSW8",
    "type":"spec/overlays/mapping/1.0",
    "attribute_mapping": {
        "email*":"email:"
    }
}
        `,
        `
{
    "capture_base":"EGKvvpidW_ytpxsdiPedznDTHAgoRL2iWBy0d2pfCSW8",
    "type":"spec/overlays/entry_code_mapping/1.0",
    "attribute_entry_codes_mapping": {
        "licenses*": [
            "A:1","B:2","C:3","D:4","E:5"
        ]
    }
}
        `
      ])
      const result = transformer.getRawDatasets()
      expect(result.length).to.be.eq(1)
      expect(result[0]).to.be.eq('email:,licenses*\ntest@example.com,["1"]')
    })
  })
})
