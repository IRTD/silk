use std::ops::Deref;

use crate::param::Param;
use serde::de::DeserializeOwned;

pub struct Json<T: DeserializeOwned + 'static + Send + Sync>(Result<T, serde_json::Error>);
impl<T> Param for Json<T>
where
    T: DeserializeOwned + 'static + Send + Sync,
{
    fn fetch(resources: &mut crate::handler::HandlerResources<'_>) -> Self {
        Json(serde_json::from_str(&resources.request.content))
    }
}

impl<T> Deref for Json<T>
where
    T: DeserializeOwned + 'static + Send + Sync,
{
    type Target = Result<T, serde_json::Error>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
