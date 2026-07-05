pub mod http;
pub mod internal;
pub mod official;
pub mod path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiSourceFamily {
    Official,
    Internal,
    Hybrid,
}
