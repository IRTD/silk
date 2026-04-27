use std::ops::Deref;

use crate::param::Param;

pub struct Path<T: PathExtractor>(T);

impl<T: PathExtractor + Send + Sync + 'static> Param for Path<T> {
    fn fetch(resources: &crate::handler::HandlerResources<'_>) -> Self {
        let raw_opt = resources.path_vars.unwrap().get(T::name());
        Path(T::parse(raw_opt))
    }
}

impl<T> Deref for Path<T>
where
    T: PathExtractor + Send + Sync,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait PathExtractor {
    fn name() -> &'static str;
    fn parse(input: Option<&String>) -> Self;
}
