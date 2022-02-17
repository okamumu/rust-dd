use crate::common::{
    HashSet,
};

pub trait Gc {
    type Node;

    fn clear_cache(&mut self);
    fn clear_table(&mut self);

    fn gc(&mut self, fs: &[&Self::Node]) {
        self.clear_cache();
        self.clear_table();
        let mut visited = HashSet::default();
        for x in fs.iter() {
            self.gc_impl(x, &mut visited);
        }
    }

    fn gc_impl(&mut self, f: &Self::Node, visited: &mut HashSet<Self::Node>);

}
