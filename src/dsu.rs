use std::collections::HashMap;
use std::hash::Hash;

pub struct Dsu<T>
where
    T: Eq + Hash + Copy,
{
    parents: HashMap<T, T>,
}

impl<T> Dsu<T>
where
    T: Eq + Hash + Copy,
{
    pub fn new() -> Self {
        Dsu {
            parents: HashMap::new(),
        }
    }

    pub fn make_set(&mut self, value: T) {
        if !self.parents.contains_key(&value) {
            self.parents.insert(value, value);
        }
    }

    pub fn find(&mut self, value: T) -> T {
        if *self.parents.get(&value).unwrap() != value {
            let root = self.find(*self.parents.get(&value).unwrap());
            self.parents.insert(value, root);
        }

        *self.parents.get(&value).unwrap()
    }

    pub fn union(&mut self, a: T, b: T) {
        let ra = self.find(a);
        let rb = self.find(b);

        if ra != rb {
            self.parents.insert(rb, ra);
        }
    }

    // pub fn groups(&mut self) -> Vec<(T, Vec<T>)> {
    //     let keys = self.parents.keys().collect::<Vec<_>>();

    //     keys.iter().as_slice().chunk_by(pred)
    // }
}
