use std::collections::{BTreeMap, BTreeSet};
use std::collections::btree_map::Entry;

#[derive(Default)]
pub struct OrderedSetMultiMap<T, U> where
    U: Ord,
    T: Ord,
{
    backing_map: BTreeMap<T, BTreeSet<U>>,
}

impl<T, U> OrderedSetMultiMap<T, U> where
    T: Ord,
    U: Ord,
{
    pub fn new() -> Self {
        OrderedSetMultiMap { backing_map: Default::default() }
    }

    pub fn get(&mut self, key: T) -> &mut BTreeSet<U> {
        self.backing_map.entry(key).or_insert_with(BTreeSet::new)
    }

    pub fn insert(&mut self, key: T, value: U) {
        self.get(key).insert(value);
    }

    pub fn remove(&mut self, key: T, value: U) {
        if let Some(set) = self.backing_map.get_mut(&key) {
            set.remove(&value);

            if set.is_empty() {
                self.backing_map.remove(&key);
            }
        }
    }

    pub fn remove_first_value_for(&mut self, key: T) -> Option<U> {
        if let Some(set) = self.backing_map.get_mut(&key) {
            if let Some(element) = set.pop_first() {
                if set.is_empty() {
                    self.backing_map.remove(&key);
                }

                return Some(element);
            }
        }

        None
    }

    pub fn next_higher_key(&self, lower_bound: T) -> Option<&T> {
        let entry = self.backing_map.range(lower_bound..).next();
        if let Some((key, _)) = entry {
            Some(key)
        } else {
            None
        }
    }
}