use std::{
    any::{Any, TypeId},
    marker::PhantomData,
    ops::Deref,
    sync::Arc,
};

use crate::param::Param;

pub struct Global<T> {
    erased: Arc<Box<dyn Any + Send + Sync>>,
    _t: PhantomData<T>,
}

impl<T: Send + Sync + 'static> Param for Global<T> {
    fn fetch(resources: &crate::handler::HandlerResources<'_>) -> Self {
        let erased = resources.global.get(&TypeId::of::<T>()).cloned().unwrap();
        Global {
            erased,
            _t: PhantomData,
        }
    }
}

impl<T: 'static> Deref for Global<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.erased.downcast_ref().unwrap()
    }
}
