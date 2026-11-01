/// Full-text query against the derived event projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoreQuery {
    pub text: String,
    pub limit: usize,
}

impl Default for StoreQuery {
    fn default() -> Self {
        Self {
            text: String::new(),
            limit: 20,
        }
    }
}
