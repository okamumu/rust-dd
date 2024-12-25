use crate::common::HashSet;

use num_traits::Num;

pub trait Count {
    type NodeId;
    type T;

    fn count(&self) -> (usize, Self::T)
    where
        Self::T: Num,
    {
        let mut visited = HashSet::default();
        let edges = self.count_edge_impl(&mut visited);
        (visited.len(), edges)
    }

    fn count_edge_impl(&self, visited: &mut HashSet<Self::NodeId>) -> Self::T
    where
        Self::T: Num;
}
