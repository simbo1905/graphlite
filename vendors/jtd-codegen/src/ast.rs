/// JTD AST node types per Section 3 of the JTD Code Generation Specification.
/// These are immutable, tagged values representing compiled schema forms.
/// Used during code generation and discarded after emission.
use std::collections::BTreeMap;

/// The 12 type keywords defined in RFC 8927 Section 2.2.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeKeyword {
    Boolean,
    String,
    Timestamp,
    Int8,
    Uint8,
    Int16,
    Uint16,
    Int32,
    Uint32,
    Float32,
    Float64,
}

impl TypeKeyword {
    pub fn parse(s: &str) -> Option<TypeKeyword> {
        match s {
            "boolean" => Some(TypeKeyword::Boolean),
            "string" => Some(TypeKeyword::String),
            "timestamp" => Some(TypeKeyword::Timestamp),
            "int8" => Some(TypeKeyword::Int8),
            "uint8" => Some(TypeKeyword::Uint8),
            "int16" => Some(TypeKeyword::Int16),
            "uint16" => Some(TypeKeyword::Uint16),
            "int32" => Some(TypeKeyword::Int32),
            "uint32" => Some(TypeKeyword::Uint32),
            "float32" => Some(TypeKeyword::Float32),
            "float64" => Some(TypeKeyword::Float64),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            TypeKeyword::Boolean => "boolean",
            TypeKeyword::String => "string",
            TypeKeyword::Timestamp => "timestamp",
            TypeKeyword::Int8 => "int8",
            TypeKeyword::Uint8 => "uint8",
            TypeKeyword::Int16 => "int16",
            TypeKeyword::Uint16 => "uint16",
            TypeKeyword::Int32 => "int32",
            TypeKeyword::Uint32 => "uint32",
            TypeKeyword::Float32 => "float32",
            TypeKeyword::Float64 => "float64",
        }
    }
}

/// An immutable AST node representing one compiled schema form.
/// Section 3.1 of the spec.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// `{}` -- accepts any JSON value
    Empty,
    /// `{"ref": "..."}` -- references a definition
    Ref { name: String },
    /// `{"type": "..."}` -- type check
    Type { type_kw: TypeKeyword },
    /// `{"enum": [...]}` -- set membership
    Enum { values: Vec<String> },
    /// `{"elements": ...}` -- array with element schema
    Elements { schema: Box<Node> },
    /// `{"properties": ..., "optionalProperties": ..., "additionalProperties": ...}`
    Properties {
        required: BTreeMap<String, Node>,
        optional: BTreeMap<String, Node>,
        additional: bool,
    },
    /// `{"values": ...}` -- object with uniform value schema
    Values { schema: Box<Node> },
    /// `{"discriminator": ..., "mapping": ...}` -- tagged union
    Discriminator {
        tag: String,
        mapping: BTreeMap<String, Node>,
    },
    /// Any form + `"nullable": true`
    Nullable { inner: Box<Node> },
}

impl Node {
    /// Returns true if this is a leaf node (Type, Enum, Empty) that should be inlined.
    pub fn is_leaf(&self) -> bool {
        matches!(self, Node::Empty | Node::Type { .. } | Node::Enum { .. })
    }

    /// Returns true if this is a complex node that should become a function call.
    pub fn is_complex(&self) -> bool {
        matches!(
            self,
            Node::Properties { .. }
                | Node::Discriminator { .. }
                | Node::Elements { .. }
                | Node::Values { .. }
                | Node::Ref { .. }
        )
    }
}

/// A compiled JTD schema: root node + definitions.
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledSchema {
    pub root: Node,
    pub definitions: BTreeMap<String, Node>,
}
