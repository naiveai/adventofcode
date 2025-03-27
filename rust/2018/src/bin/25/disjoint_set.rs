use parking_lot::{RwLock, RwLockUpgradableReadGuard};
use std::{
    collections::HashSet,
    fmt,
    iter::{self, ExactSizeIterator, Extend, FromIterator, FusedIterator, TrustedLen},
    mem,
    ops::{Index, IndexMut},
};

/// Represents a disjoint set of various subsets,
/// with fast operations to join sets together.
///
/// # Example
/// ```
/// let mut ds = DisjointSet::new();
///
/// let a = ds.make_subset(1).unwrap();
/// let b = ds.make_subset(2).unwrap();
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
pub struct DisjointSet<T> {
    roots: HashSet<usize>,
    // Each elem idx corresponds to the same idx in nodes
    elems: Vec<T>,
    nodes: Vec<RwLock<Node>>,
}

#[derive(Clone, Copy)]
struct Node {
    rank: usize,
    parent_idx: usize,
    // We use this to be able to iterate on each of our subsets.
    // This creates a circular linked list of nodes.
    next: usize,
}

impl<T> DisjointSet<T> {
    /// Creates an empty `DisjointSet`.
    pub fn new() -> Self {
        Self {
            roots: HashSet::new(),
            nodes: vec![],
            elems: vec![],
        }
    }

    /// Creates a new `DisjointSet` with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            roots: HashSet::new(),
            nodes: Vec::with_capacity(capacity),
            elems: Vec::with_capacity(capacity),
        }
    }

    pub fn capacity(&self) -> usize {
        self.elems.capacity()
    }

    /// Returns the number of subsets.
    pub fn num_subsets(&self) -> usize {
        self.roots.len()
    }

    /// Returns the number of total elements in all subsets.
    pub fn num_elements(&self) -> usize {
        self.elems.len()
    }

    // Returns true if the `DisjointSet` is empty
    pub fn is_empty(&self) -> bool {
        self.num_elements() == 0
    }

    /// Clears the `DisjointSet` of all elements.
    pub fn clear(&mut self) {
        self.roots.clear();
        self.elems.clear();
        self.nodes.clear();
    }

    /// Returns true if the given element is present in the `DisjointSet`.
    pub fn contains(&self, elem: &T) -> bool
    where
        T: PartialEq,
    {
        self.elems.contains(elem)
    }

    /// Returns the index of the given element if it exists, or None otherwise.
    pub fn position(&self, elem: &T) -> Option<usize>
    where
        T: PartialEq,
    {
        self.elems.iter().position(|e| e == elem)
    }

    /// Adds a new subset with a single, given element to the `DisjointSet`.
    /// Returns an Err with the element's existing index if it was already
    /// present in any subset, otherwise returns an Ok(usize) with the new
    /// index of the element.
    pub fn make_subset(&mut self, elem: T) -> Result<usize, DuplicateElementsError>
    where
        T: PartialEq,
    {
        if let Some(existing_idx) = self.position(&elem) {
            return Err(DuplicateElementsError { existing_idx });
        }

        // This is the index where the node will be inserted,
        // thanks to the magic of zero-indexing.
        let insertion_idx = self.elems.len();

        self.elems.push(elem);

        self.nodes.push(RwLock::new(Node {
            rank: 0,
            parent_idx: insertion_idx,
            next: insertion_idx,
        }));

        self.roots.insert(insertion_idx);

        Ok(insertion_idx)
    }

    /// Add a new subset with elements from an iterator. Returns an index
    /// that serves as this subset's representative, or an Err if there were
    /// elements in the iterator that were already present in the DisjointSet,
    /// or the iteratoe was empty.
    pub fn add_subset<I: IntoIterator<Item = T>>(
        &mut self,
        iter: I,
    ) -> Result<usize, NewSubsetError>
    where
        T: PartialEq,
    {
        let mut prev_idx = None;

        for elem in iter {
            let insertion_idx = self.make_subset(elem)?;

            if let Some(prev) = prev_idx {
                self.union(prev, insertion_idx);
            }

            prev_idx = Some(insertion_idx);
        }

        Ok(self
            .find_root_idx(prev_idx.ok_or(NewSubsetError::EmptySubset)?)
            .unwrap())
    }

    /// If present, returns an immutable reference to the element at `elem_idx`.
    pub fn get(&self, elem_idx: usize) -> Option<&T> {
        self.elems.get(elem_idx)
    }

    /// If present, returns a mutable reference to the element at `elem_idx`.
    pub fn get_mut(&mut self, elem_idx: usize) -> Option<&mut T> {
        self.elems.get_mut(elem_idx)
    }

    /// Returns an `&T` iterator over all elements in the subset
    /// elem_idx belongs to, if it exists.
    pub fn get_subset(&self, elem_idx: usize) -> Option<Subset<T>> {
        Some(Subset {
            ds: self,
            set_idxs: self.get_set_idxs(elem_idx)?,
        })
    }

    /// Returns an `&mut T` iterator over all elements in the subset
    /// elem_idx belongs to, if it exists. This iterator implements
    /// [`Extend<T>`](core::iter::Extend), so you can add elements
    /// from another iterator to this subset using it.
    pub fn get_mut_subset(&mut self, elem_idx: usize) -> Option<SubsetMut<T>> {
        let set_idxs = self.get_set_idxs(elem_idx)?;

        Some(SubsetMut { ds: self, set_idxs })
    }

    /// Returns an second-order iterator of `&T` of all the subsets.
    pub fn get_all_subsets(&self) -> impl IntoIterator<Item = Subset<T>> {
        self.roots.iter().map(move |&r| self.get_subset(r).unwrap())
    }

    /// Returns a second-order iterator of `&mut T` of all the subsets.
    pub fn get_mut_all_subsets(&mut self) -> impl IntoIterator<Item = SubsetMut<T>> {
        // Clone to avoid violating aliasing rules
        let roots = self.roots.clone();

        roots.into_iter().map(move |root| {
            // SAFETY: Here we reborrow self, which has the lifetime of this
            // closure (&'1 mut self) as an &'a mut self, which is valid here because
            // there are no overlapping indexes in each subset or among subsets.
            unsafe { &mut *(self as *mut Self) }
                .get_mut_subset(root)
                .unwrap()
        })
    }

    /// Returns Some(true) if the elements at both the given indexes
    /// are in the same subset, or None of either of them aren't present altogether.
    pub fn same_set(&self, elem1_idx: usize, elem2_idx: usize) -> Option<bool> {
        // The ? ensures this'll short-circuit and return None if either of the indexes are None,
        // meaning we don't end up returning Some(true) if both elements don't exist.
        Some(self.find_root_idx(elem1_idx)? == self.find_root_idx(elem2_idx)?)
    }

    /// Performs a union for the two subsets containing the given elements.
    /// Returns Some(true) if the operation was performed, Some(false) if not,
    /// and None if either element doesn't exist.
    ///
    /// # Example
    /// ```
    /// let mut ds = DisjointSet::new();
    ///
    /// // Ommitted: adding 5 seperate elements to the set a..e
    /// # let a = ds.make_subset(1).unwrap();
    /// # let b = ds.make_subset(2).unwrap();
    /// # let c = ds.make_subset(3).unwrap();
    /// # let d = ds.make_subset(4).unwrap();
    /// # let e = ds.make_subset(5).unwrap();
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

        let [x_root, y_root] = match self.nodes.get_disjoint_mut([x_root_idx, y_root_idx]) {
            Ok(r) => r,
            // An error here means that both indexes were the same so the elements
            // were already in the same subset, so we don't need to do anything.
            Err(_) => return Some(false),
        };
        let (x_root, y_root) = (x_root.get_mut(), y_root.get_mut());

        if x_root.rank < y_root.rank {
            // Now we swap to ensure that x_root is always the one with the higher rank.
            mem::swap(&mut x_root_idx, &mut y_root_idx);
            mem::swap(x_root, y_root);
        }

        // Now x_root.rank >= y_root.rank no matter what.
        // Therefore, make X the parent of Y.
        y_root.parent_idx = x_root_idx;
        self.roots.remove(&y_root_idx);
        if x_root.rank == y_root.rank {
            x_root.rank += 1;
        }

        // Merge the two set's circular linked lists.
        mem::swap(&mut x_root.next, &mut y_root.next);

        Some(true)
    }

    /// Returns Some(true) if the element at `elem_idx` is the only element
    /// in its subset, or None if it doesn't exist.
    pub fn is_singleton(&self, elem_idx: usize) -> Option<bool> {
        Some(self.roots.contains(&elem_idx) && self.nodes.get(elem_idx)?.read().next == elem_idx)
    }

    /// Removes the elem at `elem_idx` from its current set and into
    /// a singleton subset with only that element. Returns Some(true) if the
    /// operation was performed, Some(false) if it didn't need to be,
    /// or None if the element doesn't exist.
    pub fn make_singleton(&mut self, elem_idx: usize) -> Option<bool> {
        if self.is_singleton(elem_idx)? {
            return Some(false);
        }

        let set_idxs = self.get_set_idxs(elem_idx)?;

        let (&next_idx, &prev_idx) = set_idxs.get(1).zip(set_idxs.last()).unwrap();

        if prev_idx != elem_idx {
            let prev = self.nodes[prev_idx].get_mut();
            prev.next = next_idx;
        }

        let node = self.nodes[elem_idx].get_mut();

        self.roots.insert(elem_idx);
        node.parent_idx = elem_idx;
        node.next = elem_idx;

        Some(true)
    }

    /// Returns the index of the root of the subset
    /// `elem_idx` belongs to, if it exists.
    pub fn find_root_idx(&self, elem_idx: usize) -> Option<usize> {
        if self.roots.contains(&elem_idx) {
            return Some(elem_idx);
        }

        let mut curr_idx = elem_idx;
        let mut curr = self.nodes.get(curr_idx)?.upgradable_read();

        while curr.parent_idx != curr_idx {
            let parent_idx = curr.parent_idx;
            let parent = self.nodes[parent_idx].upgradable_read();

            // We only need a write lock for this next step.
            let mut curr_mut = RwLockUpgradableReadGuard::upgrade(curr);

            // Set the current node's parent to its grandparent.
            // This is called *path splitting*: (see the Wikipedia
            // page for details) a simpler to implement, one-pass
            // version of path compression that also, apparently,
            // turns out to be more efficient in practice.
            curr_mut.parent_idx = parent.parent_idx;

            drop(curr_mut);

            // Move up a level for the next iteration
            curr_idx = parent_idx;
            curr = parent;
        }

        Some(curr_idx)
    }

    /// Returns the indexes of all the items in the subset
    /// `elem_idx` belongs to in next-link order, if it exists.
    fn get_set_idxs(&self, elem_idx: usize) -> Option<Vec<usize>> {
        let mut curr_idx = elem_idx;
        let mut curr = self.nodes.get(curr_idx)?.read();

        let mut set_idxs = Vec::with_capacity(curr.rank);

        // We can't check the condition up here using while because
        // that would make it so the last node is never pushed.
        loop {
            set_idxs.push(curr_idx);

            // This is the last node because we've looped
            // back around to where we started.
            if curr.next == elem_idx {
                break;
            }

            curr_idx = curr.next;
            curr = self.nodes[curr.next].read();
        }

        set_idxs.shrink_to_fit();

        Some(set_idxs)
    }
}

impl<T: Eq + Clone> Clone for DisjointSet<T> {
    fn clone(&self) -> Self {
        // Node is Copy, so this should be a pretty cheap operation.
        let copied_nodes = self
            .nodes
            .iter()
            .map(|node_rwlock| RwLock::new(*node_rwlock.read()))
            .collect();

        Self {
            roots: self.roots.clone(),
            elems: self.elems.clone(),
            nodes: copied_nodes,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.roots.clone_from(&source.roots);
        self.elems.clone_from(&source.elems);

        self.nodes.resize_with(source.num_elements(), || {
            // Temporary sentinel value.
            RwLock::new(Node {
                rank: 0,
                parent_idx: 0,
                next: 0,
            })
        });

        for (source_node, dest_node) in source.nodes.iter().zip(self.nodes.iter_mut()) {
            dest_node.get_mut().clone_from(&source_node.read());
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Attempted to add a duplicate element to a DisjointSet: already existed at {existing_idx}")]
pub struct DuplicateElementsError {
    existing_idx: usize,
}

#[derive(thiserror::Error, Debug)]
pub enum NewSubsetError {
    #[error(transparent)]
    DuplicateElement(#[from] DuplicateElementsError),
    #[error("Subsets must contain at least one element")]
    EmptySubset,
}

pub struct Subset<'a, T> {
    ds: &'a DisjointSet<T>,
    set_idxs: Vec<usize>,
}

impl<'a, T> Subset<'a, T> {
    fn get(&self, index: usize) -> Option<&T> {
        Some(&self.ds[*self.set_idxs.get(index)?])
    }
}

impl<'a, T> Index<usize> for Subset<'a, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect(&format!(
            "Invalid index: the len is {} but the index is {}",
            self.set_idxs.len(),
            index
        ))
    }
}

impl<'a, T> IntoIterator for Subset<'a, T> {
    type Item = &'a T;
    type IntoIter = SubsetIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        SubsetIter {
            ds: self.ds,
            set_idxs: self.set_idxs,
            position: 0,
        }
    }
}

pub struct SubsetIter<'a, T> {
    ds: &'a DisjointSet<T>,
    set_idxs: Vec<usize>,
    position: usize,
}

impl<'a, T> Iterator for SubsetIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= self.set_idxs.len() {
            return None;
        }

        let next = self.ds.get(self.set_idxs[self.position]).unwrap();

        self.position += 1;

        Some(next)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.set_idxs.len();

        (len, Some(len))
    }
}

impl<'a, T> ExactSizeIterator for SubsetIter<'a, T> {}

unsafe impl<'a, T> TrustedLen for SubsetIter<'a, T> {}

impl<'a, T> FusedIterator for SubsetIter<'a, T> {}

pub struct SubsetMut<'a, T> {
    ds: &'a mut DisjointSet<T>,
    set_idxs: Vec<usize>,
}

impl<'a, T> SubsetMut<'a, T> {
    fn get(&self, index: usize) -> Option<&T> {
        Some(&self.ds[*self.set_idxs.get(index)?])
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        Some(&mut self.ds[*self.set_idxs.get(index)?])
    }
}

impl<'a, T> Index<usize> for SubsetMut<'a, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect(&format!(
            "Invalid index: the len is {} but the index is {}",
            self.set_idxs.len(),
            index
        ))
    }
}

impl<'a, T> IndexMut<usize> for SubsetMut<'a, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let len = self.set_idxs.len();

        self.get_mut(index).expect(&format!(
            "Invalid index: the len is {} but the index is {}",
            len, index
        ))
    }
}

impl<'a, T> IntoIterator for SubsetMut<'a, T> {
    type Item = &'a mut T;
    type IntoIter = SubsetMutIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        SubsetMutIter {
            ds: self.ds,
            set_idxs: self.set_idxs,
            position: 0,
        }
    }
}

pub struct SubsetMutIter<'a, T> {
    ds: &'a mut DisjointSet<T>,
    set_idxs: Vec<usize>,
    position: usize,
}

impl<'a, T> Iterator for SubsetMutIter<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= self.set_idxs.len() {
            return None;
        }

        let next = unsafe {
            // SAFETY: We're in a &mut DisjointSet context, so the current
            // thread has exclusive access, and there are no duplicate
            // indexes in the set.
            &mut *(self.ds.get_mut(self.set_idxs[self.position]).unwrap() as *mut _)
        };

        self.position += 1;

        Some(next)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.set_idxs.len();

        (len, Some(len))
    }
}

impl<'a, T> ExactSizeIterator for SubsetMutIter<'a, T> {}

unsafe impl<'a, T> TrustedLen for SubsetMutIter<'a, T> {}

impl<'a, T> FusedIterator for SubsetMutIter<'a, T> {}

impl<'a, T: PartialEq> Extend<T> for SubsetMut<'a, T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let set_representative = self.set_idxs[0];

        for elem in iter {
            let insertion_idx = match self.ds.make_subset(elem) {
                Ok(idx) => idx,
                Err(e @ DuplicateElementsError { existing_idx }) => {
                    if self.set_idxs.contains(&existing_idx) {
                        // Already contained in the current set, ignore.
                        continue;
                    } else {
                        panic!("{}. Use DisjointSet::union to merge two subsets.", e);
                    }
                }
            };

            self.set_idxs.push(insertion_idx);
            self.ds.union(set_representative, insertion_idx);
        }
    }

    fn extend_reserve(&mut self, additional: usize) {
        self.ds.extend_reserve(additional);
    }
}

impl<T: fmt::Debug> fmt::Debug for DisjointSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?}",
            self.into_iter()
                .map(|subset| subset.into_iter().collect::<Vec<_>>())
                .collect::<Vec<_>>()
        )
    }
}

impl<T> Default for DisjointSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Index<usize> for DisjointSet<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect(&format!(
            "index out of bounds: the len is {} but the index is {}",
            self.num_elements(),
            index
        ))
    }
}

impl<T> IndexMut<usize> for DisjointSet<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let len = self.num_elements();

        self.get_mut(index).expect(&format!(
            "index out of bounds: the len is {} but the index is {}",
            len, index
        ))
    }
}

impl<T: PartialEq> PartialEq for DisjointSet<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.num_subsets() != other.num_subsets() || self.num_elements() != other.num_elements()
        {
            return false;
        }

        for (self_subset, other_subset) in self.into_iter().zip(other.into_iter()) {
            let mut other_subset = other_subset.into_iter();

            for elem in self_subset {
                if let Some(other_elem) = other_subset.next() {
                    if elem != other_elem {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }

        true
    }
}

impl<T: Eq> Eq for DisjointSet<T> {}

impl<T: PartialEq> Extend<T> for DisjointSet<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.add_subset(iter).unwrap();
    }

    fn extend_one(&mut self, item: T) {
        self.make_subset(item).unwrap();
    }

    fn extend_reserve(&mut self, additional: usize) {
        self.elems.extend_reserve(additional);
        self.nodes.extend_reserve(additional);
    }
}

impl<T: PartialEq, I: IntoIterator<Item = T>> FromIterator<I> for DisjointSet<T> {
    fn from_iter<II: IntoIterator<Item = I>>(iter: II) -> Self {
        let mut ds = Self::new();

        for subset in iter {
            let mut subset = subset.into_iter();

            let set_representative = ds
                .make_subset(match subset.next() {
                    Some(elem) => elem,
                    None => continue,
                })
                .unwrap();

            ds.get_mut_subset(set_representative)
                .unwrap()
                .extend(subset);
        }

        ds
    }
}

impl<T> From<DisjointSet<T>> for Vec<Vec<T>> {
    default fn from(ds: DisjointSet<T>) -> Self {
        let all_sets_idxs = ds
            .roots
            .iter()
            .map(|&root| ds.get_set_idxs(root).unwrap())
            .collect::<Vec<_>>();

        let mut vec_2d: Vec<Vec<T>> = iter::repeat_with(Vec::new)
            .take(all_sets_idxs.len())
            .collect();

        for (i, elem) in ds.elems.into_iter().enumerate() {
            vec_2d[all_sets_idxs.iter().position(|v| v.contains(&i)).unwrap()].push(elem);
        }

        vec_2d
    }
}

// This is possible because of the "specialization" feature
// on the crate level. I wanted to do this because there's
// a more efficient way to accomplish this conversion
// if T: Default.
impl<T: Default> From<DisjointSet<T>> for Vec<Vec<T>> {
    fn from(mut ds: DisjointSet<T>) -> Self {
        (&mut ds)
            .into_iter()
            .map(|set_iter| {
                set_iter
                    .into_iter()
                    .map(|elem| {
                        // Replace each element with its default to
                        // keep everything valid while we're iterating.
                        // ds is gonna get dropped anyway.
                        mem::take(elem)
                    })
                    .collect()
            })
            .collect()
    }
}

impl<T> IntoIterator for DisjointSet<T> {
    type Item = impl ExactSizeIterator<Item = T> + DoubleEndedIterator;
    type IntoIter = impl ExactSizeIterator<Item = Self::Item> + DoubleEndedIterator;

    fn into_iter(self) -> Self::IntoIter {
        <Vec<Vec<_>>>::from(self).into_iter().map(|v| v.into_iter())
    }
}

impl<'a, T> IntoIterator for &'a DisjointSet<T> {
    type Item = Subset<'a, T>;
    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.get_all_subsets().into_iter()
    }
}

impl<'a, T> IntoIterator for &'a mut DisjointSet<T> {
    type Item = SubsetMut<'a, T>;
    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.get_mut_all_subsets().into_iter()
    }
}
