# OCA conductor

## Introduction

One of the fundamental concepts behind OCA is data harmonization. To harmonize data is to unify two or more datasets to be compliant to each other so that further processing can be performed on unified data. Harmonization may take place on the dataset level, where as a result all the datasets become unified to the same schema. Another type of harmonization is to achieve a compliant subset among schemas. The process to achieve harmonization is called a transformation. Transformation overlays are the recipe to how actually data (or schema) should be transformed. 

`OCA conductor` serves dual purpose:
- data validation where provided data record is verified against schema defined within `OCA Bundle`;
- data transformation where injected datasets are transformed to the new state using transformation overlays.

`OCA conductor` is written in Rust thanks to which most C-based languages can be supported by exposing proper FFI-layer. We provide bindings for the following languages:
- [Typescript/Javascript](/bindings/node.js)

## License

EUPL 1.2 

We have distilled the most crucial license specifics to make your adoption seamless: [see here for details](https://github.com/THCLab/licensing).

## Installation

### Rust
```
[dependencies]
oca_conductor = "0.2.12"
```
### Typescript and JavaScript (Node.JS based)

- For `oca-transformer`
  ```
  npm i oca-data-transformer
  ```
- For `oca-validator`
  ```
  npm i oca-data-validator
  ```

### Python (validator)

> Prerequisites:  
> Install Rust - https://www.rust-lang.org

Clone repository and go to `bindings/ffi/validator` directory

```
git clone https://github.com/THCLab/oca-conductor.git
cd bindings/ffi/validator
```

Run
```
cargo build --release --features validator
cargo run --bin uniffi-bindgen generate src/validator.udl -l python
cp target/release/libvalidator.so src/libuniffi_validator.so
```

All you need are `validator.py` and `libuniffi_validator.so` files from `src` directory.

Example usage:
```python
from validator import *

oca_str = """
{
  "capture_base": {
    "attributes": {
      "n1": "Text",
      "n2": "Numeric"
    },
    "classification": "",
    "digest": "ElNWOR0fQbv_J6EL0pJlvCxEpbu4bg1AurHgr_0A7LKc",
    "flagged_attributes": [],
    "type": "spec/capture_base/1.0"
  },
  "overlays": [
    {
      "attribute_entry_codes": {
        "n1": ["op1", "op2"]
      },
      "capture_base": "ElNWOR0fQbv_J6EL0pJlvCxEpbu4bg1AurHgr_0A7LKc",
      "digest": "E4L-BukSBsqZoDDIJvw4_gGjAJs5It4UUfiA200lGup0",
      "type": "spec/overlays/entry_code/1.0"
    },
    {
      "attribute_conformance": {
        "n1": "M"
      },
      "capture_base": "ElNWOR0fQbv_J6EL0pJlvCxEpbu4bg1AurHgr_0A7LKc",
      "digest": "E4L-BukSBsqZoDDIJvw4_gGjAJs5It4UUfiA200lGup0",
      "type": "spec/overlays/conformance/1.0"
    }
  ]
}
"""

validator = Validator(oca_str)

validator.set_constraints(ConstraintsConfig(fail_on_additional_attributes=True))

try:
    validator.validate(
    """
    [
        {
            "n1": "op",
            "additional": 1
        },
        {
            "n2": 2
        }
    ]
    """
    )
except ValidationErrors.List as error_list:
    for error in error_list.errors:
        print(error)

# Result:
# ValidationError.Other(data_set='0', record='0', attribute_name='n1', message='\'n1\' value ("op") must be one of ["op1", "op2"]')
# ValidationError.Other(data_set='0', record='0', attribute_name='additional', message='unknown_attribute')
# ValidationError.Other(data_set='0', record='1', attribute_name='n1', message='missing_attribute')
```

## Transformation overlays

### Attribute Mapping Overlay

An Attribute Mapping Overlay defines attribute mappings between two distinct data models. The example below shows the schema manipulation of the source attribute name and the destination attribute name.

```json
  {
    "attribute_mapping": {
      "firstName": "first_name",
      "surname": "last_name"
    },
    "capture_base": "EzMTX96ZkwRBzkUBI9c8jiXfHa6CfKJyrd6uH6vjIUZc",
    "digest": "EBfxGsWV4aQGRfRsVH_tDfQCFaBnNqA7cq8YW1Q3DdwA",
    "type": "spec/overlays/mapping/1.0"
  }
```

### Entry Mapping Overlay

An Entry Code Mapping Overlay defines entry key mappings between two distinct code tables (aka lookup tables) or datasets. The example below shows the mapping between three letter country codes into two letter country codes. 

```json
  {
    "attribute_entry_codes_mapping": {
      "country_code": [
        "AF:AFG","AL:ALB","DZ:DZA","AS:ASM","AD:AND","AO:AGO","AI:AIA","AQ:ATA","AG:ATG","AR:ARG","AM:ARM","AW:ABW","AU:AUS","AT:AUT","AZ:AZE","BS:BHS","BH:BHR","BD:BGD","BB:BRB","BY:BLR","BE:BEL","BZ:BLZ","BJ:BEN","BM:BMU","BT:BTN","BO:BOL","BQ:BES","BA:BIH","BW:BWA","BV:BVT","BR:BRA","IO:IOT","BN:BRN","BG:BGR","BF:BFA","BI:BDI","CV:CPV","KH:KHM","CM:CMR","CA:CAN","KY:CYM","CF:CAF","TD:TCD","CL:CHL","CN:CHN","CX:CXR","CC:CCK","CO:COL","KM:COM","CD:COD","CG:COG","CK:COK","CR:CRI","HR:HRV","CU:CUB","CW:CUW","CY:CYP","CZ:CZE","CI:CIV","DK:DNK","DJ:DJI","DM:DMA","DO:DOM","EC:ECU","EG:EGY","SV:SLV","GQ:GNQ","ER:ERI","EE:EST","SZ:SWZ","ET:ETH","FK:FLK","FO:FRO","FJ:FJI","FI:FIN","FR:FRA","GF:GUF","PF:PYF","TF:ATF","GA:GAB","GM:GMB","GE:GEO","DE:DEU","GH:GHA","GI:GIB","GR:GRC","GL:GRL","GD:GRD","GP:GLP","GU:GUM","GT:GTM","GG:GGY","GN:GIN","GW:GNB","GY:GUY","HT:HTI","HM:HMD","VA:VAT","HN:HND","HK:HKG","HU:HUN","IS:ISL","IN:IND","ID:IDN","IR:IRN","IQ:IRQ","IE:IRL","IM:IMN","IL:ISR","IT:ITA","JM:JAM","JP:JPN","JE:JEY","JO:JOR","KZ:KAZ","KE:KEN","KI:KIR","KP:PRK","KR:KOR","KW:KWT","KG:KGZ","LA:LAO","LV:LVA","LB:LBN","LS:LSO","LR:LBR","LY:LBY","LI:LIE","LT:LTU","LU:LUX","MO:MAC","MG:MDG","MW:MWI","MY:MYS","MV:MDV","ML:MLI","MT:MLT","MH:MHL","MQ:MTQ","MR:MRT","MU:MUS","YT:MYT","MX:MEX","FM:FSM","MD:MDA","MC:MCO","MN:MNG","ME:MNE","MS:MSR","MA:MAR","MZ:MOZ","MM:MMR","NA:NAM","NR:NRU","NP:NPL","NL:NLD","NC:NCL","NZ:NZL","NI:NIC","NE:NER","NG:NGA","NU:NIU","NF:NFK","MP:MNP","NO:NOR","OM:OMN","PK:PAK","PW:PLW","PS:PSE","PA:PAN","PG:PNG","PY:PRY","PE:PER","PH:PHL","PN:PCN","PL:POL","PT:PRT","PR:PRI","QA:QAT","MK:MKD","RO:ROU","RU:RUS","RW:RWA","RE:REU","BL:BLM","SH:SHN","KN:KNA","LC:LCA","MF:MAF","PM:SPM","VC:VCT","WS:WSM","SM:SMR","ST:STP","SA:SAU","SN:SEN","RS:SRB","SC:SYC","SL:SLE","SG:SGP","SX:SXM","SK:SVK","SI:SVN","SB:SLB","SO:SOM","ZA:ZAF","GS:SGS","SS:SSD","ES:ESP","LK:LKA","SD:SDN","SR:SUR","SJ:SJM","SE:SWE","CH:CHE","SY:SYR","TW:TWN","TJ:TJK","TZ:TZA","TH:THA","TL:TLS","TG:TGO","TK:TKL","TO:TON","TT:TTO","TN:TUN","TR:TUR","TM:TKM","TC:TCA","TV:TUV","UG:UGA","UA:UKR","AE:ARE","GB:GBR","UM:UMI","US:USA","UY:URY","UZ:UZB","VU:VUT","VE:VEN","VN:VNM","VG:VGB","VI:VIR","WF:WLF","EH:ESH","YE:YEM","ZM:ZMB","ZW:ZWE","AX:ALA"
      ]
    },
    "capture_base": "EzMTX96ZkwRBzkUBI9c8jiXfHa6CfKJyrd6uH6vjIUZc",
    "digest": "EnBjuRoJs8TmfCQJdF8BdmmHwEUbCRrw45xwIFCS7qdY",
    "type": "spec/overlays/entry_code_mapping/1.0"
  }
```

### Unit Mapping Overlay

A Unit Mapping Overlay defines target units when converting between different units of measurement. The example below demands measurement recalculation into `mmol/L` unit, where the source unit is known from the Unit Overlay of the OCA Bundle. 

```json
  {
    "attribute_units": {
      "blood_glucose": "mmol/L"
    },
    "capture_base": "EzMTX96ZkwRBzkUBI9c8jiXfHa6CfKJyrd6uH6vjIUZc",
    "digest": "EwpEtyZgBF0B9KXEufQQdeS2aN1GL4JIMcpVRffH3_pg",
    "metric_system": "nonSI",
    "type": "spec/overlays/unit/1.0"
  }
```

