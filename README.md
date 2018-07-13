\# sion-rs

[SION](https://dankogai.github.io/SION/) serializer/deserializer implementation in Rust.

## How to use
Add the following to your `Cargo.toml`
```toml
[dependencies]
serde = "1.0"
serde_derive = "1.0"
sion-rs = { git = "https://github.com/pandaman64/sion-rs.git" }
```

Then you can use serde's `Serialize`/`Deserialize` to deal with SION data.
```rust
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate sion_rs;

use sion_rs::{to_string, from_str};

fn main() {
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct Person {
        name: String,
        age: u32,
    }

    let person = Person {
        name: "dankogai".into(),
        age: 48,
    };
    let expected = r#"["name":"dankogai","age":48]"#;
    assert_eq!(to_string(&person).unwrap(), expected);

    let deserialized = from_str::<Person>(expected).unwrap();
    assert_eq!(deserialized, person);
}

```

## Future work
- [ ] handle `.Date` correctly (now `sion-rs` treats any `.Date` as a single float, collapsing the structure)
