use super::*;

#[test]
fn starts_with_every_index_in_its_own_set() {
    let mut sets = DisjointSet::new(3);
    assert_eq!(sets.find(0), 0);
    assert_eq!(sets.find(1), 1);
    assert_eq!(sets.find(2), 2);
}

#[test]
fn union_puts_indices_in_the_same_set() {
    let mut sets = DisjointSet::new(3);
    sets.union(0, 1);
    assert_eq!(sets.find(0), sets.find(1));
    assert_ne!(sets.find(0), sets.find(2));
}

#[test]
fn union_is_transitive() {
    let mut sets = DisjointSet::new(4);
    sets.union(0, 1);
    sets.union(2, 3);
    sets.union(1, 2);
    assert_eq!(sets.find(0), sets.find(3));
}

#[test]
fn repeated_union_changes_nothing() {
    let mut sets = DisjointSet::new(2);
    sets.union(0, 1);
    sets.union(1, 0);
    assert_eq!(sets.find(0), sets.find(1));
}
