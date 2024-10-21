use std::sync::Arc;

#[derive(Clone)]
pub struct Storage {
    inner: Arc<StorageInner>,
}

pub struct StorageInner {}

impl std::ops::Deref for Storage {
    type Target = StorageInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
