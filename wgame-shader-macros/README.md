# wgame-shader-macros

Procedural macros for deriving shader attribute implementations.

Provides `#[derive(Attribute)]` and `#[derive(AttributeGlobal)]` for structs.

## Features

- `Attribute` derive for regular shader attributes with prefixed bindings
- `AttributeGlobal` derive for global uniforms without prefixing
- Automatic binding generation for struct fields
- Support for glam and primitive types

