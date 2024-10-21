use std::sync::Arc;

#[derive(Clone)]
pub struct Ingest {
    inner: Arc<IngestInner>,
}

struct IngestInner {}
