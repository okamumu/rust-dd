use crate::common::BddHashSet;
use crate::common::NodeId;
use std::io::BufWriter;

pub trait Dot {
    type Node;

    fn dot<T>(&self, io: &mut T, node: &Self::Node)
    where
        T: std::io::Write,
    {
        let s1 = "digraph { layout=dot; overlap=false; splines=true; node [fontsize=10];\n";
        let s2 = "}\n";
        let mut visited: BddHashSet<NodeId> = BddHashSet::default();
        io.write_all(s1.as_bytes()).unwrap();
        self.dot_impl(io, node, &mut visited);
        io.write_all(s2.as_bytes()).unwrap();
    }

    fn dot_string(&self, node: &Self::Node) -> String {
        let mut buf = vec![];
        {
            let mut io = BufWriter::new(&mut buf);
            self.dot(&mut io, node);
        }
        std::str::from_utf8(&buf).unwrap().to_string()
    }

    fn dot_impl<T>(&self, io: &mut T, node: &Self::Node, visited: &mut BddHashSet<NodeId>)
    where
        T: std::io::Write;
}
