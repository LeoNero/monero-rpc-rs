use {
    failure::Fallible,
    serde::{Deserialize, Deserializer, Serialize},
    std::fmt::{self, Display},
};

pub trait HashType: Sized {
    fn bytes(&self) -> &[u8];
    fn from_str(v: &str) -> Fallible<Self>;
}

macro_rules! hash_type_impl {
    ($name:ty) => {
        impl HashType for $name {
            fn bytes(&self) -> &[u8] {
                self.as_bytes()
            }
            fn from_str(v: &str) -> ::failure::Fallible<Self> {
                Ok(v.parse()?)
            }
        }
    };
}

hash_type_impl!(monero::PaymentId);
hash_type_impl!(monero::cryptonote::hash::Hash);

impl HashType for Vec<u8> {
    fn bytes(&self) -> &[u8] {
        &*self
    }
    fn from_str(v: &str) -> Fallible<Self> {
        Ok(hex::decode(v)?)
    }
}

#[derive(Clone, Debug)]
pub struct HashString<T>(pub T);

impl<T> Display for HashString<T>
where
    T: HashType,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0.bytes()))
    }
}

impl<'a, T> Serialize for HashString<T>
where
    T: HashType,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de, T> Deserialize<'de> for HashString<T>
where
    T: HashType,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self(T::from_str(&s).map_err(serde::de::Error::custom)?))
    }
}
