pub struct CacheConfig {
    pub node_cache_size: usize,
    pub edge_cache_size: usize,
}

impl CacheConfig {
    pub fn new(node_cache_size: usize, edge_cache_size: usize) -> Self {
        Self {
            node_cache_size,
            edge_cache_size,
        }
    }

    pub fn none() -> Self {
        Self::new(0, 0)
    }
}
