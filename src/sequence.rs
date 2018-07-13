use value::Value;

// FIXME: insertion order
#[derive(Debug, PartialEq)]
pub struct Map {
    pub(crate) contents: Vec<(Value, Value)>,
}

#[derive(Debug, PartialEq)]
pub struct Array {
    pub(crate) contents: Vec<Value>,
}
