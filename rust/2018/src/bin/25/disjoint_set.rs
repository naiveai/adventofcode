use std::{
    cell::RefCell,
    collections::HashSet,
    convert::TryFrom,
    iter::{self, ExactSizeIterator},
    mem,
};

/// Represents a disjoint set of various subsets,
/// with fast operations to join sets together.
///
/// # Example
/// ```
/// let mut ds = DisjointSet::new();
///
/// let a = ds.make_set(1).unwrap();
/// let b = ds.make_set(2).unwrap();
///
/// assert!(ds.contains(&1) && ds.contains(&2));
/// assert_eq!(ds.same_set(a, b), Some(false));
/// assert_eq!(ds.num_sets(), 2);
///
/// assert_eq!(ds.union(a, b), Some(true));
///
/// assert_eq!(ds.same_set(a, b), Some(true));
/// assert_eq!(ds.num_sets(), 1);
/// ```
// Details about the algorithm used here can be found
// at the Wikipedia page for "Disjoint-set data structure".
#[derive(Clone)]
pub struct DisjointSet<T: Eq> {
    /// Contains the root indexes for each set.
    /// Can be used as a "representative" value
    /// for methods on `DisjointSet`.
    roots: HashSet<usize>,
    nodes: Vec<RefCell<Node<T>>>,
}

#[derive(Clone)]
struct Node<T> {
    elem: T,
    parent_idx: usize,
    rank: usize,
    // We use this to be able to iterate
    // on each of our subsets.
    next: usize,
}

impl<T: Eq> DisjointSet<T> {
    /// Creates an empty `DisjointSet`.
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            roots: HashSet::new(),
        }
    }

    /// Creates a new `DisjointSet` with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
            roots: HashSet::new(),
        }
    }

    /// Adds a new set with a single, given element to
    /// the `DisjointSet`. Returns an Err with the elem
    /// if it was already present in any set, otherwise
    /// returns a Ok(usize) with the index of the element.
    pub fn make_set(&mut self, elem: T) -> Result<usize, T> {
        if !self.contains(&elem) {
            // This is the index where the node will be inserted,
            // thanks to the magic of zero-indexing.
            let insertion_idx = self.nodes.len();

            self.nodes.push(RefCell::new(Node {
                elem,
                parent_idx: insertion_idx,
                rank: 0,
                next: insertion_idx,
            }));

            self.roots.insert(insertion_idx);

            Ok(insertion_idx)
        } else {
            Err(elem)
        }
    }

    /// Returns true if the given element is present in the `DisjointSet`.
    pub fn contains(&self, elem: &T) -> bool {
        self.position(elem).is_some()
    }

    /// Returns the index of the given element if it exists, or None otherwise.
    pub fn position(&self, elem: &T) -> Option<usize> {
        self.nodes.iter().position(|e| &e.borrow().elem == elem)
    }

    /// If present, returns an immutable reference to the element at `elem_idx`.
    ///
    /// This requires &mut self because that ensures you have a lock and
    /// can't use other methods that may use interior mutability.
    pub fn get(&mut self, elem_idx: usize) -> Option<&T> {
        self.get_mut(elem_idx).map(|e| &*e)
    }

    /// If present, returns a mutable reference to the element at `elem_idx`.
    pub fn get_mut(&mut self, elem_idx: usize) -> Option<&mut T> {
        let node_cell = self.nodes.get_mut(elem_idx)?;

        // RefCall::get_mut is used rarely, but here it's appropriate:
        // As long as the &mut T from this is still in scope,
        // the caller won't be able to use any other methods,
        // so interior mutability isn't a concern.
        Some(&mut node_cell.get_mut().elem)
    }

    /// Returns the number of subsets.
    pub fn num_sets(&self) -> usize {
        self.roots.len()
    }

    /// Returns the number of total elements in all subsets.
    pub fn num_elements(&self) -> usize {
        self.nodes.len()
    }

    /// Returns an `&T` iterator over all elements in the set
    /// elem_idx belongs to, if it exists.
    ///
    /// This requires `&mut self` because that ensures you have a lock
    /// and can't use other methods that may use interior mutability.
    pub fn iter_set(
        &mut self,
        elem_idx: usize,
    ) -> Option<impl ExactSizeIterator<Item = &T> + DoubleEndedIterator<Item = &T>> {
        self.iter_mut_set(elem_idx).map(|v| v.map(|e| &*e))
    }

    /// Returns an `&mut T` iterator over all elements in the set
    /// elem_idx belongs to, if it exists.
    // We use both applicable Iterator types here to give the caller
    // the maximum possible flexbility when using the returned value.
    pub fn iter_mut_set(
        &mut self,
        elem_idx: usize,
    ) -> Option<impl ExactSizeIterator<Item = &mut T> + DoubleEndedIterator<Item = &mut T>> {
        let set_idxs = self.get_set_idxs(elem_idx)?;

        Some(
            set_idxs
                .iter()
                .map(|&i| {
                    // In reality this is safe because there'll
                    // be no duplicate indexes. But Rust doesn't
                    // have any way of knowing that.
                    unsafe { &mut *(self.get_mut(i).unwrap() as *mut _) }
                })
                // We have to collect in between to ensure
                // the closure that borrows self doesn't need
                // to leave the function.
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }

    /// Returns Some(true) if the elements at both the given indexes
    /// are in the same set, or None of either of them aren't present altogether.
    pub fn same_set(&self, elem1_idx: usize, elem2_idx: usize) -> Option<bool> {
        // The ? ensures this'll short-circuit and return None if either of the indexes are None,
        // meaning we don't end up returning Some(true) if both elements don't exist.
        Some(self.find_root_idx(elem1_idx)? == self.find_root_idx(elem2_idx)?)
    }

    /// Performs a union for the two sets containing the given elements.
    /// Returns Some(true) if the operation was performed, Some(false) if not,
    /// and None if either element doesn't exist.
    ///
    /// # Example
    /// ```
    /// let mut ds = DisjointSet::new();
    ///
    /// // Ommitted: adding 5 seperate elements to the set a..e
    /// # let a = ds.make_set(1).unwrap();
    /// # let b = ds.make_set(2).unwrap();
    /// # let c = ds.make_set(3).unwrap();
    /// # let d = ds.make_set(4).unwrap();
    /// # let e = ds.make_set(5).unwrap();
    ///
    /// assert_eq!(ds.union(a, b), Some(true));
    ///
    /// assert_eq!(ds.same_set(a, b), Some(true));
    /// assert_eq!(ds.num_sets(), 4);
    ///
    /// assert_eq!(ds.union(a, b), Some(false));
    /// assert_eq!(ds.union(c, d), Some(true));
    /// assert_eq!(ds.union(e, c), Some(true));
    ///
    /// // Now we have {a, b} and {c, d, e}
    ///
    /// assert_eq!(ds.num_sets(), 2);
    /// assert_eq!(ds.same_set(a, c), Some(false));
    /// assert_eq!(ds.same_set(d, e), Some(true));
    ///
    /// assert_eq!(ds.union(a, e), Some(true));
    ///
    /// assert_eq!(ds.num_sets(), 1);
    /// ```
    pub fn union(&mut self, elem_x_idx: usize, elem_y_idx: usize) -> Option<bool> {
        let (mut x_root_idx, mut y_root_idx) = (
            self.find_root_idx(elem_x_idx)?,
            self.find_root_idx(elem_y_idx)?,
        );

        // We don't have to do anything if this is the case.
        // Also, if we didn't check this, we'd panic below because
        // we'd attempt two mutable borrowings of the same RefCell.
        if x_root_idx == y_root_idx {
            return Some(false);
        }

        let (mut x_root, mut y_root) = (
            self.nodes[x_root_idx].borrow_mut(),
            self.nodes[y_root_idx].borrow_mut(),
        );

        if x_root.rank < y_root.rank {
            // Must use mem::swap here. If we shadowed,
            // it'd go out of scope when the if block ended.
            mem::swap(&mut x_root_idx, &mut y_root_idx);
            mem::swap(&mut x_root, &mut y_root);
        }

        // Now x_root.rank >= y_root.rank no matter what.
        // Therefore, make X the parent of Y.
        y_root.parent_idx = x_root_idx;
        self.roots.remove(&y_root_idx);
        if x_root.rank == y_root.rank {
            x_root.rank += 1;
        }

        // Drop the RefMuts so we can check self.last,
        // which needs to immutably borrow, without conflicts.
        drop(x_root);
        drop(y_root);

        let x_last_idx = self.last(x_root_idx).unwrap();
        let mut x_last = self.nodes[x_last_idx].borrow_mut();
        x_last.next = y_root_idx;

        Some(true)
    }

    /// Returns the index of the root of the subset
    /// `elem_idx` belongs to, if it exists.
    pub fn find_root_idx(&self, elem_idx: usize) -> Option<usize> {
        if self.roots.contains(&elem_idx) {
            return Some(elem_idx);
        }

        let mut curr_idx = elem_idx;
        let mut curr = self.nodes.get(curr_idx)?.borrow_mut();

        while curr.parent_idx != curr_idx {
            let parent_idx = curr.parent_idx;
            let parent = self.nodes[parent_idx].borrow_mut();

            // Set the current node's parent to its grandparent.
            // This is called *path splitting*: (see the Wikipedia
            // page for details) a simpler to implement, one-pass
            // version of path compression that also, apparently,
            // turns out to be more efficient in practice.
            curr.parent_idx = parent.parent_idx;

            // Move up a level for the next iteration
            curr_idx = parent_idx;
            curr = parent;
        }

        Some(curr_idx)
    }

    /// Returns the last element of the subset with
    /// `elem_idx` in it, if it exists.
    fn last(&self, elem_idx: usize) -> Option<usize> {
        self.get_set_idxs(elem_idx)?.pop()
    }

    /// Returns the indexes of all the items in the subset
    /// `elem_idx` belongs to in arbitrary order, if it exists.
    fn get_set_idxs(&self, elem_idx: usize) -> Option<Vec<usize>> {
        let mut curr_idx = self.find_root_idx(elem_idx)?;
        let mut curr = self.nodes[curr_idx].borrow();

        let mut set_idxs = Vec::with_capacity(self.num_elements());

        // We can't check the condition up here
        // using while because that would make
        // it so the last node is never pushed.
        loop {
            set_idxs.push(curr_idx);

            // This is the last node
            if curr_idx == curr.next {
                break;
            }

            curr_idx = curr.next;
            curr = self.nodes[curr.next].borrow();
        }

        set_idxs.shrink_to_fit();

        Some(set_idxs)
    }
}

impl<T: Eq> TryFrom<Vec<T>> for DisjointSet<T> {
    type Error = TryFromVecError;

    fn try_from(vec: Vec<T>) -> Result<Self, Self::Error> {
        use TryFromVecError::*;

        let mut ds = Self::with_capacity(vec.len());

        for elem in vec {
            ds.make_set(elem).map_err(|_| DuplicateElements)?;
        }

        Ok(ds)
    }
}

impl<T: Eq> TryFrom<Vec<Vec<T>>> for DisjointSet<T> {
    type Error = TryFromVecError;

    fn try_from(mut vec_2d: Vec<Vec<T>>) -> Result<Self, Self::Error> {
        use TryFromVecError::*;

        let mut ds = Self::with_capacity(vec_2d.iter().map(|v| v.len()).sum());

        let mut ds_indexes = Vec::with_capacity(vec_2d.len());

        for vec in &mut vec_2d {
            if vec.is_empty() {
                continue;
            }

            ds_indexes.push(
                ds.make_set(vec.swap_remove(0))
                    .map_err(|_| DuplicateElements)?,
            );
        }

        for (i, vec) in vec_2d.into_iter().enumerate() {
            for elem in vec {
                let elem_idx = ds.make_set(elem).map_err(|_| DuplicateElements)?;

                // Add the element to its corresponding set
                ds.union(ds_indexes[i], elem_idx);
            }
        }

        Ok(ds)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum TryFromVecError {
    #[error("Two duplicate elements were found.")]
    DuplicateElements,
}

impl<T: Eq> From<DisjointSet<T>> for Vec<Vec<T>> {
    default fn from(ds: DisjointSet<T>) -> Self {
        let mut all_sets_idxs = vec![];

        for &root in ds.roots.iter() {
            all_sets_idxs.push(ds.get_set_idxs(root).unwrap());
        }

        let mut vec_2d: Vec<Vec<T>> = iter::repeat_with(Vec::new)
            .take(all_sets_idxs.len())
            .collect();

        for (i, node) in ds.nodes.into_iter().enumerate() {
            vec_2d[all_sets_idxs.iter().position(|v| v.contains(&i)).unwrap()]
                .push(node.into_inner().elem);
        }

        vec_2d
    }
}

// This is possible because of the "specialization" feature
// on the crate level. I wanted to do this because there's
// a more efficient way to accomplish this conversion
// if T: Default.
impl<T: Eq + Default> From<DisjointSet<T>> for Vec<Vec<T>> {
    fn from(mut ds: DisjointSet<T>) -> Self {
        let mut all_sets_idxs = vec![];

        for &root in ds.roots.iter() {
            all_sets_idxs.push(ds.get_set_idxs(root).unwrap());
        }

        all_sets_idxs
            .into_iter()
            .map(|set_idx_vec| {
                set_idx_vec
                    .into_iter()
                    .map(|i| {
                        // Replace each element with its default to keep everything
                        // valid while we're iterating. ds is gonna get dropped anyway.
                        mem::take(ds.get_mut(i).unwrap())
                    })
                    .collect()
            })
            .collect()
    }
}

impl<T: Eq> IntoIterator for DisjointSet<T> {
    type Item = impl ExactSizeIterator<Item = T> + DoubleEndedIterator<Item = T>;
    type IntoIter = impl ExactSizeIterator<Item = Self::Item> + DoubleEndedIterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        <Vec<Vec<T>>>::from(self).into_iter().map(|v| v.into_iter())
    }
}