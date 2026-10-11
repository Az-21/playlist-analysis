//! Union-find clustering shared by the similarity analyses

/// Union-find over item indices
#[derive(Debug, Default, Clone)]
pub(super) struct DisjointSet {
    parents: Vec<usize>,
}

impl DisjointSet {
    /// Creates a set for every index in `0..len`
    pub(super) fn new(len: usize) -> Self {
        Self {
            parents: (0..len).collect(),
        }
    }

    /// Returns the representative of the set containing `index`
    pub(super) fn find(&mut self, index: usize) -> usize {
        let mut root = index;
        while self.parents[root] != root {
            root = self.parents[root];
        }
        let mut current = index;
        while self.parents[current] != root {
            let next = self.parents[current];
            self.parents[current] = root;
            current = next;
        }
        root
    }

    /// Merges the sets containing `left` and `right`
    pub(super) fn union(&mut self, left: usize, right: usize) {
        let left_root = self.find(left);
        let right_root = self.find(right);
        if left_root != right_root {
            self.parents[right_root] = left_root;
        }
    }
}

#[cfg(test)]
mod tests;
