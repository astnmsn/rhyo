use std::{
    fmt::{Debug, Display},
    hash::Hash,
};

pub mod slotted_page;

pub trait KeyType: Debug + Display + Clone + Eq + PartialOrd + Hash + Send + Default {}

pub trait ValueType: Debug + Display + Clone + Hash + Send + Default {}

pub type Tuple<K: KeyType, V: ValueType> = (K, V);

// impl<T: Debug + Display + Clone + Eq + PartialOrd + Hash + Send + Default> KeyType for T {}
// impl<T: Debug + Display + Clone + Hash + Send + Default> ValueType for T {}
