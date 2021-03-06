/**
 * A doubly-linked list. Supports O(1) head, tail, count, push, pop, etc.
 *
 * Do not use ==, !=, <, etc on doubly-linked lists -- it may not terminate.
 */

export dlist, dlist_node;
export create, from_elt, from_vec, extensions;

type dlist_link<T> = option<dlist_node<T>>;

enum dlist_node<T> = @{
    data: T,
    mut root: option<dlist<T>>,
    mut prev: dlist_link<T>,
    mut next: dlist_link<T>
};

// Needs to be an @-box so nodes can back-reference it.
enum dlist<T> = @{
    mut size: uint,
    mut hd: dlist_link<T>,
    mut tl: dlist_link<T>
};

impl private_methods<T> for dlist_node<T> {
    pure fn assert_links() {
        alt self.next {
            some(neighbour) {
                alt neighbour.prev {
                    some(me) {
                        if !box::ptr_eq(*self, *me) {
                            fail "Asymmetric next-link in dlist node."
                        }
                    }
                    none { fail "One-way next-link in dlist node." }
                }
            }
            none { }
        }
        alt self.prev {
            some(neighbour) {
                alt neighbour.next {
                    some(me) {
                        if !box::ptr_eq(*me, *self) {
                            fail "Asymmetric prev-link in dlist node."
                        }
                    }
                    none { fail "One-way prev-link in dlist node." }
                }
            }
            none { }
        }
    }
}

impl extensions<T> for dlist_node<T> {
    /// Get the next node in the list, if there is one.
    pure fn next_link() -> option<dlist_node<T>> {
        self.assert_links();
        self.next
    }
    /// Get the next node in the list, failing if there isn't one.
    pure fn next_node() -> dlist_node<T> {
        alt self.next_link() {
            some(nobe) { nobe }
            none       { fail "This dlist node has no next neighbour." }
        }
    }
    /// Get the previous node in the list, if there is one.
    pure fn prev_link() -> option<dlist_node<T>> {
        self.assert_links();
        self.prev
    }
    /// Get the previous node in the list, failing if there isn't one.
    pure fn prev_node() -> dlist_node<T> {
        alt self.prev_link() {
            some(nobe) { nobe }
            none       { fail "This dlist node has no previous neighbour." }
        }
    }

    /// Remove a node from whatever dlist it's on (failing if none).
    fn remove() {
        if option::is_some(self.root) {
            option::get(self.root).remove(self);
        } else {
            fail "Removing an orphaned dlist node - what do I remove from?"
        }
    }
}

/// Creates a new dlist node with the given data.
pure fn create_node<T>(+data: T) -> dlist_node<T> {
    dlist_node(@{data: data, mut root: none, mut prev: none, mut next: none})
}

/// Creates a new, empty dlist.
pure fn create<T>() -> dlist<T> {
    dlist(@{mut size: 0, mut hd: none, mut tl: none})
}

/// Creates a new dlist with a single element
fn from_elt<T>(+data: T) -> dlist<T> {
    let list = create();
    list.push(data);
    list
}

fn from_vec<T: copy>(+vec: &[T]) -> dlist<T> {
    do vec::foldl(create(), vec) |list,data| {
        list.push(data); // Iterating left-to-right -- add newly to the tail.
        list
    }
}

impl private_methods<T> for dlist<T> {
    pure fn new_link(-data: T) -> dlist_link<T> {
        some(dlist_node(@{data: data, mut root: some(self),
                          mut prev: none, mut next: none}))
    }
    pure fn assert_mine(nobe: dlist_node<T>) {
        alt nobe.root {
            some(me) { assert box::ptr_eq(*self, *me); }
            none     { fail "This node isn't on this dlist." }
        }
    }
    fn make_mine(nobe: dlist_node<T>) {
        if option::is_some(nobe.root) {
            fail "Cannot insert node that's already on a dlist!"
        }
        nobe.root = some(self);
    }
    // Link two nodes together. If either of them are 'none', also sets
    // the head and/or tail pointers appropriately.
    #[inline(always)]
    fn link(+before: dlist_link<T>, +after: dlist_link<T>) {
        alt before {
            some(neighbour) { neighbour.next = after; }
            none            { self.hd        = after; }
        }
        alt after {
            some(neighbour) { neighbour.prev = before; }
            none            { self.tl        = before; }
        }
    }
    // Remove a node from the list.
    fn unlink(nobe: dlist_node<T>) {
        self.assert_mine(nobe);
        assert self.size > 0;
        self.link(nobe.prev, nobe.next);
        nobe.prev = none; // Release extraneous references.
        nobe.next = none;
        nobe.root = none;
        self.size -= 1;
    }

    fn add_head(+nobe: dlist_link<T>) {
        self.link(nobe, self.hd); // Might set tail too.
        self.hd = nobe;
        self.size += 1;
    }
    fn add_tail(+nobe: dlist_link<T>) {
        self.link(self.tl, nobe); // Might set head too.
        self.tl = nobe;
        self.size += 1;
    }
    fn insert_left(nobe: dlist_link<T>, neighbour: dlist_node<T>) {
        self.assert_mine(neighbour);
        assert self.size > 0;
        self.link(neighbour.prev, nobe);
        self.link(nobe, some(neighbour));
        self.size += 1;
    }
    fn insert_right(neighbour: dlist_node<T>, nobe: dlist_link<T>) {
        self.assert_mine(neighbour);
        assert self.size > 0;
        self.link(nobe, neighbour.next);
        self.link(some(neighbour), nobe);
        self.size += 1;
    }
}

impl extensions<T> for dlist<T> {
    /// Get the size of the list. O(1).
    pure fn len()          -> uint { self.size }
    /// Returns true if the list is empty. O(1).
    pure fn is_empty()     -> bool { self.len() == 0 }
    /// Returns true if the list is not empty. O(1).
    pure fn is_not_empty() -> bool { self.len() != 0 }

    /// Add data to the head of the list. O(1).
    fn push_head(+data: T) {
        self.add_head(self.new_link(data));
    }
    /**
     * Add data to the head of the list, and get the new containing
     * node. O(1).
     */
    fn push_head_n(+data: T) -> dlist_node<T> {
        let mut nobe = self.new_link(data);
        self.add_head(nobe);
        option::get(nobe)
    }
    /// Add data to the tail of the list. O(1).
    fn push(+data: T) {
        self.add_tail(self.new_link(data));
    }
    /**
     * Add data to the tail of the list, and get the new containing
     * node. O(1).
     */
    fn push_n(+data: T) -> dlist_node<T> {
        let mut nobe = self.new_link(data);
        self.add_tail(nobe);
        option::get(nobe)
    }
    /**
     * Insert data into the middle of the list, left of the given node.
     * O(1).
     */
    fn insert_before(+data: T, neighbour: dlist_node<T>) {
        self.insert_left(self.new_link(data), neighbour);
    }
    /**
     * Insert an existing node in the middle of the list, left of the
     * given node. O(1).
     */
    fn insert_n_before(nobe: dlist_node<T>, neighbour: dlist_node<T>) {
        self.make_mine(nobe);
        self.insert_left(some(nobe), neighbour);
    }
    /**
     * Insert data in the middle of the list, left of the given node,
     * and get its containing node. O(1).
     */
    fn insert_before_n(+data: T, neighbour: dlist_node<T>) -> dlist_node<T> {
        let mut nobe = self.new_link(data);
        self.insert_left(nobe, neighbour);
        option::get(nobe)
    }
    /**
     * Insert data into the middle of the list, right of the given node.
     * O(1).
     */
    fn insert_after(+data: T, neighbour: dlist_node<T>) {
        self.insert_right(neighbour, self.new_link(data));
    }
    /**
     * Insert an existing node in the middle of the list, right of the
     * given node. O(1).
     */
    fn insert_n_after(nobe: dlist_node<T>, neighbour: dlist_node<T>) {
        self.make_mine(nobe);
        self.insert_right(neighbour, some(nobe));
    }
    /**
     * Insert data in the middle of the list, right of the given node,
     * and get its containing node. O(1).
     */
    fn insert_after_n(+data: T, neighbour: dlist_node<T>) -> dlist_node<T> {
        let mut nobe = self.new_link(data);
        self.insert_right(neighbour, nobe);
        option::get(nobe)
    }

    /// Remove a node from the head of the list. O(1).
    fn pop_n() -> option<dlist_node<T>> {
        let hd = self.peek_n();
        hd.map(|nobe| self.unlink(nobe));
        hd
    }
    /// Remove a node from the tail of the list. O(1).
    fn pop_tail_n() -> option<dlist_node<T>> {
        let tl = self.peek_tail_n();
        tl.map(|nobe| self.unlink(nobe));
        tl
    }
    /// Get the node at the list's head. O(1).
    pure fn peek_n() -> option<dlist_node<T>> { self.hd }
    /// Get the node at the list's tail. O(1).
    pure fn peek_tail_n() -> option<dlist_node<T>> { self.tl }

    /// Get the node at the list's head, failing if empty. O(1).
    pure fn head_n() -> dlist_node<T> {
        alt self.hd {
            some(nobe) { nobe }
            none       { fail "Attempted to get the head of an empty dlist." }
        }
    }
    /// Get the node at the list's tail, failing if empty. O(1).
    pure fn tail_n() -> dlist_node<T> {
        alt self.tl {
            some(nobe) { nobe }
            none       { fail "Attempted to get the tail of an empty dlist." }
        }
    }

    /// Remove a node from anywhere in the list. O(1).
    fn remove(nobe: dlist_node<T>) { self.unlink(nobe); }

    /// Check data structure integrity. O(n).
    fn assert_consistent() {
        if option::is_none(self.hd) || option::is_none(self.tl) {
            assert option::is_none(self.hd) && option::is_none(self.tl);
        }
        // iterate forwards
        let mut count = 0;
        let mut link = self.peek_n();
        let mut rabbit = link;
        while option::is_some(link) {
            let nobe = option::get(link);
            // check self on this list
            assert option::is_some(nobe.root) &&
                box::ptr_eq(*option::get(nobe.root), *self);
            // check cycle
            if option::is_some(rabbit) { rabbit = option::get(rabbit).next; }
            if option::is_some(rabbit) { rabbit = option::get(rabbit).next; }
            if option::is_some(rabbit) {
                assert !box::ptr_eq(*option::get(rabbit), *nobe);
            }
            // advance
            link = nobe.next_link();
            count += 1;
        }
        assert count == self.len();
        // iterate backwards - some of this is probably redundant.
        link = self.peek_tail_n();
        rabbit = link;
        while option::is_some(link) {
            let nobe = option::get(link);
            // check self on this list
            assert option::is_some(nobe.root) &&
                box::ptr_eq(*option::get(nobe.root), *self);
            // check cycle
            if option::is_some(rabbit) { rabbit = option::get(rabbit).prev; }
            if option::is_some(rabbit) { rabbit = option::get(rabbit).prev; }
            if option::is_some(rabbit) {
                assert !box::ptr_eq(*option::get(rabbit), *nobe);
            }
            // advance
            link = nobe.prev_link();
            count -= 1;
        }
        assert count == 0;
    }
}

impl extensions<T: copy> for dlist<T> {
    /// Remove data from the head of the list. O(1).
    fn pop()       -> option<T> { self.pop_n().map       (|nobe| nobe.data) }
    /// Remove data from the tail of the list. O(1).
    fn pop_tail()  -> option<T> { self.pop_tail_n().map  (|nobe| nobe.data) }
    /// Get data at the list's head. O(1).
    fn peek()      -> option<T> { self.peek_n().map      (|nobe| nobe.data) }
    /// Get data at the list's tail. O(1).
    fn peek_tail() -> option<T> { self.peek_tail_n().map (|nobe| nobe.data) }
    /// Get data at the list's head, failing if empty. O(1).
    pure fn head() -> T         { self.head_n().data }
    /// Get data at the list's tail, failing if empty. O(1).
    pure fn tail() -> T         { self.tail_n().data }
}
