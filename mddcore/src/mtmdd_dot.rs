use crate::mtmdd::*;
use crate::nodes::*;
use common::prelude::*;

impl<V> Dot for MtMddManager<V>
where
    V: MddValue,
{
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
                    if let Node::Terminal(_) | Node::NonTerminal(_) = self.get_node(xid).unwrap() {
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
