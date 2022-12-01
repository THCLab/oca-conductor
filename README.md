# OCA conductor

## Introduction

One of the fundamental concepts behind OCA is data harmonization. To harmonize data is to unify two or more datasets to be compliant to each other so that further processing can be performed on unified data. Harmonization may take place on the dataset level, where as a result all the datasets become unified to the same schema. Another type of harmonization is to achieve a compliant subset among schemas. The process to achieve harmonization is called a transformation. Transformation overlays are the recipe to how actually data (or schema) should be transformed. 

`OCA conductor` serves dual purpose:
- data validation where provided data record is verified against schema defined within `OCA Bundle`;
- data transformation where injected datasets are transformed to the new state using transformation overlays.

`OCA conductor` is written in Rust thanks to which most C-based languages can be supported by exposing proper FFI-layer. We provide bindings for the following languages:
- [Typescript/Javascript](/bindings/node.js)

## Installation

### Rust
```
[dependencies]
oca_conductor = "0.2.7"
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

## Transformation overlays

### Attribute Mapping Overlay

An Attribute Mapping Overlay defines attribute mappings between two distinct data models. The example below shows the schema manipulation of the source attribute name and the destination attribute name.

```
 {
    "capture_base":"Ev_RaB-gIOn8VAB3sg40mINxjiYRxdLVQrgce0aZbFcc",
    "type":"spec/overlays/mapping/1.0",
    "attribute_mapping":{
      "first_name":"firstName",
      "last_name":"surname"
    }
  }
```

### Entry Mapping Overlay

An Entry Code Mapping Overlay defines entry key mappings between two distinct code tables (aka lookup tables) or datasets. The example below shows the mapping between three letter country codes into two letter country codes. 

```
  {
    "capture_base":"Ev_RaB-gIOn8VAB3sg40mINxjiYRxdLVQrgce0aZbFcc",
    "type":"spec/overlays/entry_code_mapping/1.0",
    "attribute_entry_codes_mapping":{
      "country_code":["AFG:AF","ALB:AL","DZA:DZ","ASM:AS","AND:AD","AGO:AO","AIA:AI","ATA:AQ","ATG:AG","ARG:AR","ARM:AM","ABW:AW","AUS:AU","AUT:AT","AZE:AZ","BHS:BS","BHR:BH","BGD:BD","BRB:BB","BLR:BY","BEL:BE","BLZ:BZ","BEN:BJ","BMU:BM","BTN:BT","BOL:BO","BES:BQ","BIH:BA","BWA:BW","BVT:BV","BRA:BR","IOT:IO","BRN:BN","BGR:BG","BFA:BF","BDI:BI","CPV:CV","KHM:KH","CMR:CM","CAN:CA","CYM:KY","CAF:CF","TCD:TD","CHL:CL","CHN:CN","CXR:CX","CCK:CC","COL:CO","COM:KM","COD:CD","COG:CG","COK:CK","CRI:CR","HRV:HR","CUB:CU","CUW:CW","CYP:CY","CZE:CZ","CIV:CI","DNK:DK","DJI:DJ","DMA:DM","DOM:DO","ECU:EC","EGY:EG","SLV:SV","GNQ:GQ","ERI:ER","EST:EE","SWZ:SZ","ETH:ET","FLK:FK","FRO:FO","FJI:FJ","FIN:FI","FRA:FR","GUF:GF","PYF:PF","ATF:TF","GAB:GA","GMB:GM","GEO:GE","DEU:DE","GHA:GH","GIB:GI","GRC:GR","GRL:GL","GRD:GD","GLP:GP","GUM:GU","GTM:GT","GGY:GG","GIN:GN","GNB:GW","GUY:GY","HTI:HT","HMD:HM","VAT:VA","HND:HN","HKG:HK","HUN:HU","ISL:IS","IND:IN","IDN:ID","IRN:IR","IRQ:IQ","IRL:IE","IMN:IM","ISR:IL","ITA:IT","JAM:JM","JPN:JP","JEY:JE","JOR:JO","KAZ:KZ","KEN:KE","KIR:KI","PRK:KP","KOR:KR","KWT:KW","KGZ:KG","LAO:LA","LVA:LV","LBN:LB","LSO:LS","LBR:LR","LBY:LY","LIE:LI","LTU:LT","LUX:LU","MAC:MO","MDG:MG","MWI:MW","MYS:MY","MDV:MV","MLI:ML","MLT:MT","MHL:MH","MTQ:MQ","MRT:MR","MUS:MU","MYT:YT","MEX:MX","FSM:FM","MDA:MD","MCO:MC","MNG:MN","MNE:ME","MSR:MS","MAR:MA","MOZ:MZ","MMR:MM","NAM:NA","NRU:NR","NPL:NP","NLD:NL","NCL:NC","NZL:NZ","NIC:NI","NER:NE","NGA:NG","NIU:NU","NFK:NF","MNP:MP","NOR:NO","OMN:OM","PAK:PK","PLW:PW","PSE:PS","PAN:PA","PNG:PG","PRY:PY","PER:PE","PHL:PH","PCN:PN","POL:PL","PRT:PT","PRI:PR","QAT:QA","MKD:MK","ROU:RO","RUS:RU","RWA:RW","REU:RE","BLM:BL","SHN:SH","KNA:KN","LCA:LC","MAF:MF","SPM:PM","VCT:VC","WSM:WS","SMR:SM","STP:ST","SAU:SA","SEN:SN","SRB:RS","SYC:SC","SLE:SL","SGP:SG","SXM:SX","SVK:SK","SVN:SI","SLB:SB","SOM:S​​O","ZAF:ZA","SGS:GS","SSD:SS","ESP:ES","LKA:LK","SDN:SD","SUR:SR","SJM:SJ","SWE:SE","CHE:CH","SYR:SY","TWN:TW","TJK:TJ","TZA:TZ","THA:TH","TLS:TL","TGO:TG","TKL:TK","TON:TO","TTO:TT","TUN:TN","TUR:TR","TKM:TM","TCA:TC","TUV:TV","UGA:UG","UKR:UA","ARE:AE","GBR:GB","UMI:UM","USA:US","URY:UY","UZB:UZ","VUT:VU","VEN:VE","VNM:VN","VGB:VG","VIR:VI","WLF:WF","ESH:EH","YEM:YE","ZMB:ZM","ZWE:ZW","ALA:AX"
      ]
    }
  }
```

### Unit Mapping Overlay

A Unit Mapping Overlay defines target units when converting between different units of measurement. The example below demands measurement recalculation into `mg/dL` unit, where the source unit is known from the Unit Overlay of the OCA Bundle. 

```
  {
    "capture_base":"Ev_RaB-gIOn8VAB3sg40mINxjiYRxdLVQrgce0aZbFcc",
    "type":"spec/overlays/unit/1.0",
    "metric_system":"SI",
    "attribute_units":{
      "blood_glucose":"mg/dL"
    }
  }
```

