use crate::common::{
    HashSet,
};

pub trait Dot {
    type Node;

    fn dot<T>(&self, io: &mut T) where T: std::io::Write {
        let s1 = "digraph { layout=dot; overlap=false; splines=true; node [fontsize=10];\n";
        let s2 = "}\n";
        let mut visited: HashSet<Self::Node> = HashSet::new();
        io.write(s1.as_bytes()).unwrap();
        self.dot_impl(io, &mut visited);
        io.write(s2.as_bytes()).unwrap();
    }

    fn dot_impl<T>(&self, io: &mut T, visited: &mut HashSet<Self::Node>) where T: std::io::Write;
}
