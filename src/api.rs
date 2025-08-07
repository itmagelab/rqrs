//! This is module documentation for api
//!
//! A model for connecting to basic REST API.
//! A description of all available entry points can be

#[allow(dead_code)]
static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[allow(dead_code)]
trait Rq {
    fn apply_if<T, F>(self, val: Option<T>, fun: F) -> Self
    where
        Self: Sized,
        F: FnOnce(Self, T) -> Self,
    {
        if let Some(val) = val {
            fun(self, val)
        } else {
            self
        }
    }
}
