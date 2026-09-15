#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MeshHandle(usize);

impl MeshHandle {
    pub(crate) fn new(index: usize) -> Self {
        Self(index)
    }

    pub(crate) fn index(self) -> usize {
        self.0
    }
}
