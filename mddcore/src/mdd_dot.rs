use crate::mdd::*;
use common::prelude::*;

impl Dot for MddManager {
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
            Node::Zero => {
                let s = format!("\"obj{}\" [shape=square, label=\"F\"];\n", id);
                io.write_all(s.as_bytes()).unwrap();
            }
            Node::One => {
                let s = format!("\"obj{}\" [shape=square, label=\"T\"];\n", id);
                io.write_all(s.as_bytes()).unwrap();
            }
            Node::Undet => {
                let s = format!("\"obj{}\" [shape=square, label=\"?\"];\n", id);
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
                    if let Node::Zero | Node::One | Node::NonTerminal(_) =
                        self.get_node(xid).unwrap()
                    {
                        self.dot_impl(io, xid, visited);
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
