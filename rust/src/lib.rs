pub mod traits;
pub mod r#enum;
pub mod error;
pub mod nats;

pub use r#enum::Format;
pub use error::Error;
pub use traits::{PubTrait, SubTrait};

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct MyObj {
        field: String,
    }

    #[tokio::test]
    async fn messagepack_roundtrip() {
        let obj = MyObj { field: "value".into() };
        let data = rmp_serde::to_vec(&obj).unwrap();
        let decoded: MyObj = rmp_serde::from_slice(&data).unwrap();
        assert_eq!(obj, decoded);
    }
}
