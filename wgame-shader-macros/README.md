# wgame-shader-macros

Derive vertex/instance attribute serialization for named and tuple structs.
`Attribute` uses the `wgame_shader` dependency path; `AttributeGlobal` uses
`wgame::shader`. Both prefix binding names with their field names.

See [shader attributes](../docs/GUIDE.md#custom-rendering-and-shader-attributes)
for layout rules and limitations. These are not uniform-buffer packing derives.
