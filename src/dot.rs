use std::io::BufWriter;
use crate::common::HashSet;

pub trait Dot {
    type Node;

    fn dot<T>(&self, io: &mut T) where T: std::io::Write {
        let s1 = "digraph { layout=dot; overlap=false; splines=true; node [fontsize=10];\n";
        let s2 = "}\n";
        let mut visited: HashSet<Self::Node> = HashSet::default();
        io.write(s1.as_bytes()).unwrap();
        self.dot_impl(io, &mut visited);
        io.write(s2.as_bytes()).unwrap();
    }

    fn dot_string(&self) -> String {
        let mut buf = vec![];
        {
            let mut io = BufWriter::new(&mut buf);
            self.dot(&mut io);
        }
        std::str::from_utf8(&buf).unwrap().to_string()
    }
    
    fn dot_impl<T>(&self, io: &mut T, visited: &mut HashSet<Self::Node>) where T: std::io::Write;
}
