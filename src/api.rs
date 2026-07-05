pub mod http;
pub mod internal;
pub mod official;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiSourceFamily {
    Official,
    Internal,
    Hybrid,
}
