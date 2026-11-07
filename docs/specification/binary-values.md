# Binary values

`Value::Bytes(Vec<u8>)` is an in-memory value only in version 0.1.x. It has no
textual CBR representation: both formatters return a formatting error. This is
intentional until an encoding and syntax are standardized. Crypto APIs operate
on byte slices independently and do not encode bytes as CBR values.
