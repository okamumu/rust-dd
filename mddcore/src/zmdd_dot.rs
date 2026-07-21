//! Graphviz rendering for [`ZmddManager`].
//!
//! Unlike the MTMDD renderer, the 0-edge is **drawn**: in a ZMDD the 0-edge carries the
//! "component absent from the sparse vector" branch and dropping it would lose structure.
//! Edge labels are the raw edge indices; a `reverse` (cut) family in the `mss` layer
//! reports states as `edge_num-1 - d` when extracting, but the graph shown here is the
//! raw diagram.

use crate::mtmdd::Node;
use crate::zmdd::ZmddManager;
use crate::nodes::*;
use common::prelude::*;

impl<V> Dot for ZmddManager<V>
where
    V: MddValue,
{
    type Node = NodeId;

    fn dot_impl<T>(&self, io: &mut T, id: &NodeId, visited: &mut BddHashSet<NodeId>)
    where
        T: std::io::Write,
    {
        if visited.contains(id) {
            return;
        }
        let node = self.get_node(id).unwrap();
        match node {
            Node::Undet => {
                let s = format!("\"obj{}\" [shape=square, label=\"Undet\"];\n", id);
                io.write_all(s.as_bytes()).unwrap();
            }
            Node::Terminal(fnode) => {
                let s = format!(
                    "\"obj{}\" [shape=square, label=\"{}\"];\n",
                    fnode.id(),
                    fnode.value()
                );
                io.write_all(s.as_bytes()).unwrap();
            }
            Node::NonTerminal(fnode) => {
                let s = format!(
                    "\"obj{}\" [shape=circle, label=\"{}\"];\n",
                    fnode.id(),
                    self.label(id).unwrap()
                );
                io.write_all(s.as_bytes()).unwrap();
                for (i, xid) in fnode.iter().enumerate() {
                    self.dot_impl(io, &xid, visited);
                    let s = format!(
                        "\"obj{}\" -> \"obj{}\" [label=\"{}\"];\n",
                        fnode.id(),
                        xid,
                        i
                    );
                    io.write_all(s.as_bytes()).unwrap();
                }
            }
        };
        visited.insert(*id);
    }
}
