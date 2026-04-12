use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use tokio::sync::RwLock;

use crate::router::Router;

pub(crate) type GlobalMap = Arc<HashMap<TypeId, Arc<RwLock<Box<dyn Any>>>>>;

pub struct Server {
    router: Arc<Router>,
    global_map: GlobalMap,
}
