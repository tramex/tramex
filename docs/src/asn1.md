# ASN.1

ASN.1 (Abstract Syntax Notation One) is a standard interface description language used for defining data structures that can be serialized and deserialized in a cross-platform way. It is used in telecommunications, cryptography, and other fields.

## Download ASN.1 from 3gpp docx

- go to <https://www.3gpp.org/specifications-technologies/specifications-by-series>
- click on the LTE series (36 series) <https://portal.3gpp.org/Specifications.aspx?q=1&series=30&releases=all&draft=False&underCC=False&withACC=False&withBCC=False&numberNYA=False>
- click on the specification you want to download
- a window will pop up, go to the "Release" tab
- click on the release version to download the file (zip)

Another solution is to access through the ftp server:

- <https://www.3gpp.org/ftp/Specs/archive/36_series/>

## Convert docx to asn1

If the document from 3gpp has asn1 inside, it can be extracted with the following command (using the tool [docx-asn1]( https://github.com/its-just-nans/docx-asn1)):

```sh
# download docx from 3gpp

# install python package docx_asn1
python -m pip install docx_asn1

# use docx_asn1 to convert docx to asn1
python -m docx_asn1 file.docx > output.asn1

# the converted asn1 file is then called output.asn1 and can be found in the current directory
```

## asn1c

For raw decoding, the tool `asn1c` (<https://github.com/vlm/asn1c>) can be used to generate C code from the ASN.1 file. The generated code can be used to decode the messages.

## `types_lte_3gpp`

The crate `types_lte_3gpp` (<https://github.com/tramex/types_lte_3gpp/>) is used to automatically generate Rust code from ASN.1 files (extracted automatically from docx files created by 3gpp). It is used in the Tramex project to generate the code for the LTE protocol.
