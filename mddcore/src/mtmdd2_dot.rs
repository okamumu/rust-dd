use common::prelude::*;
use crate::nodes::*;
use crate::mtmdd2::*;

impl<V> Dot for MtMdd2Manager<V>
where
    V: MddValue
{
    type Node = Node;

    fn dot_impl<T>(&self, io: &mut T, node: &Node, visited: &mut BddHashSet<NodeId>)
    where
        T: std::io::Write,
    {
        match node {
            Node::Value(f) => self.mtmdd().dot_impl(io, f, visited),
            Node::Bool(f) => self.mdd().dot_impl(io, f, visited),
        }
    }
}
