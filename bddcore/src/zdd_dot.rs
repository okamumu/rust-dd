//! Graphviz rendering for [`ZddManager`].
//!
//! The `0` terminal (the empty family) and every edge into it are **omitted** — they carry
//! no information and clutter the picture. `0` is therefore drawn only when it is the root
//! itself (so an empty family does not render as an empty graph). The 0-*edge* is still
//! drawn wherever it leads somewhere: in a ZDD it means "element not in the set".

use common::prelude::*;
use crate::nodes::*;
use crate::zdd::*;

impl Dot for ZddManager {
    type Node = NodeId;

    fn dot_impl<T>(&self, io: &mut T, id: &NodeId, visited: &mut BddHashSet<NodeId>)
    where
        T: std::io::Write,
    {
        if visited.contains(&id) {
            return;
        }
        let node = self.get_node(id).unwrap();
        match node {
            Node::Undet => {
                let s = format!("\"obj{}\" [shape=square, label=\"*\"];\n", id);
                io.write_all(s.as_bytes()).unwrap();
            }
            Node::Zero => {
                let s = format!("\"obj{}\" [shape=square, label=\"0\"];\n", id);
                io.write_all(s.as_bytes()).unwrap();
            }
            Node::One => {
                let s = format!("\"obj{}\" [shape=square, label=\"1\"];\n", id);
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
                    // The `0` terminal is the empty family: skip the node and the arrow.
                    // (The 0-*edge* itself is kept — it means "element not in the set".)
                    if let Node::One | Node::NonTerminal(_) = self.get_node(&xid).unwrap() {
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
            }
        };
        visited.insert(*id);
    }
}
