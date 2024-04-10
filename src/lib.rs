#[macro_export]
// Taken from https://docs.rs/map-macro/latest/src/map_macro/lib.rs.html#140-144
macro_rules! hash_set {
    {$($v: expr),* $(,)?} => {
        std::collections::HashSet::from([$($v,)*])
    };
}

pub mod discreet;
pub mod continuous;