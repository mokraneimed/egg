use crate::Id;
use std::sync::RwLock;
use std::fmt::Debug;

#[derive(Debug, Default)]
pub struct UnionFind {
    parents: RwLock<Vec<Id>>, // RwLock to allow safe concurrent access
}

impl Clone for UnionFind {
    fn clone(&self) -> Self {
        UnionFind {
            parents: RwLock::new(self.parents.read().unwrap().clone()), // Clone the inner Vec
        }
    }
}

impl UnionFind {
    pub fn make_set(&self) -> Id {
        let mut parents = self.parents.write().unwrap(); // Write access to modify
        let id = Id::from(parents.len());
        parents.push(id);
        id
    }

    #[inline(always)]
    fn parent(&self, query: Id) -> Id {
        let parents = self.parents.read().unwrap(); // Read access for querying
        parents[usize::from(query)]
    }

    #[inline(always)]
    fn set_parent(&self, query: Id, new_parent: Id) {
        let mut parents = self.parents.write().unwrap(); // Write access to modify
        parents[usize::from(query)] = new_parent;
    }

    pub fn find(&self, mut current: Id) -> Id {
        loop {
            let parent = self.parent(current);
            if current == parent {
                return parent;
            }
            // do path halving and proceed
            let grandparent = self.parent(parent);
            self.set_parent(current, grandparent);
            current = grandparent;
        }
    }

    /// Returns (new_leader, old_leader)
    pub fn union(&self, set1: Id, set2: Id) -> (Id, Id) {
        let mut root1 = self.find(set1);
        let mut root2 = self.find(set2);

        if root1 == root2 {
            (root1, root2)
        } else {
            if root1 > root2 {
                // NOTE egg actually relied on the returned id being the minimum
                std::mem::swap(&mut root1, &mut root2);
            }
            self.set_parent(root2, root1);
            (root1, root2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use indexmap::{indexmap, indexset, IndexMap, IndexSet};

    impl UnionFind {
        pub fn build_sets(&self) -> IndexMap<Id, IndexSet<Id>> {
            let mut map: IndexMap<Id, IndexSet<Id>> = Default::default();

            for i in 0..self.parents.read().unwrap().len() {
                let i = Id::from(i);
                let leader = self.find(i);
                map.entry(leader).or_default().insert(i);
            }

            map
        }
    }

    fn make_union_find(n: usize) -> UnionFind {
        let uf = UnionFind::default();
        for _ in 0..n {
            uf.make_set();
        }
        uf
    }

    #[test]
    fn union_find() {
        let n = 10;

        fn id(u: usize) -> Id {
            u.into()
        }

        let uf = make_union_find(n);

        // test the initial condition of everyone in their own set
        for i in 0..n {
            let i = Id::from(i);
            assert_eq!(uf.find(i), i);
            assert_eq!(uf.find(i), i);
        }

        // make sure build_sets works
        let expected_sets = (0..n)
            .map(|i| (id(i), indexset!(id(i))))
            .collect::<IndexMap<_, _>>();
        assert_eq!(uf.build_sets(), expected_sets);

        // build up one set
        assert_eq!(uf.union(id(0), id(1)), (id(0), id(1)));
        assert_eq!(uf.union(id(1), id(2)), (id(0), id(2)));
        assert_eq!(uf.union(id(3), id(2)), (id(0), id(3)));

        // build up another set
        assert_eq!(uf.union(id(6), id(7)), (id(6), id(7)));
        assert_eq!(uf.union(id(8), id(9)), (id(8), id(9)));
        assert_eq!(uf.union(id(7), id(9)), (id(6), id(8)));

        // make sure union on same set returns to == from
        assert_eq!(uf.union(id(1), id(3)), (id(0), id(0)));
        assert_eq!(uf.union(id(7), id(8)), (id(6), id(6)));

        // check set structure
        let expected_sets = indexmap!(
            id(0) => indexset!(id(0), id(1), id(2), id(3)),
            id(4) => indexset!(id(4)),
            id(5) => indexset!(id(5)),
            id(6) => indexset!(id(6), id(7), id(8), id(9)),
        );
        assert_eq!(uf.build_sets(), expected_sets);

        // all paths should be compressed at this point
        for i in 0..n {
            assert_eq!(uf.parent(id(i)), uf.find(id(i)));
        }
    }
}
