use crate::exceptions::{exc_err_fmt, ExcType};
use crate::heap::{Heap, ObjectId};
use crate::object::{Attr, Object};
use crate::run::RunResult;

/// Python list type, wrapping a Vec of Objects.
///
/// This type provides Python list semantics including dynamic growth,
/// reference counting for heap objects, and standard list methods like
/// append and insert.
///
/// # Reference Counting
/// When objects are added to the list (via append, insert, etc.), their
/// reference counts are incremented if they are heap-allocated (Ref variants).
/// This ensures objects remain valid while referenced by the list.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct List(Vec<Object>);

impl List {
    /// Creates a new list from a vector of objects.
    ///
    /// Note: This does NOT increment reference counts - the caller must
    /// ensure refcounts are properly managed.
    #[must_use]
    pub fn from_vec(vec: Vec<Object>) -> Self {
        Self(vec)
    }

    /// Returns a reference to the underlying vector.
    #[must_use]
    pub fn as_vec(&self) -> &Vec<Object> {
        &self.0
    }

    /// Returns a mutable reference to the underlying vector.
    ///
    /// # Safety Considerations
    /// Be careful when mutating the vector directly - you must manually
    /// manage reference counts for any heap objects you add or remove.
    pub fn as_vec_mut(&mut self) -> &mut Vec<Object> {
        &mut self.0
    }

    /// Consumes the list and returns the underlying vector.
    #[must_use]
    pub fn into_vec(self) -> Vec<Object> {
        self.0
    }

    /// Returns the number of elements in the list.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the list is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn py_eq(&self, other: &Self, heap: &Heap) -> bool {
        self.len() == other.len() && self.0.iter().zip(&other.0).all(|(i1, i2)| i1.py_eq(i2, heap))
    }

    /// Appends an element to the end of the list.
    ///
    /// If the item is a heap-allocated object (Ref variant), its reference
    /// count is incremented automatically.
    ///
    /// Returns `Object::None`, matching Python's behavior where `list.append()` returns None.
    pub fn append(&mut self, heap: &mut Heap, item: Object) -> Object {
        // Increment refcount if item is heap-allocated
        if let Object::Ref(item_id) = &item {
            heap.inc_ref(*item_id);
        }
        self.0.push(item);
        Object::None
    }

    /// Inserts an element at the specified index.
    ///
    /// If the item is a heap-allocated object (Ref variant), its reference
    /// count is incremented automatically.
    ///
    /// # Arguments
    /// * `index` - The position to insert at (0-based). If index >= len(),
    ///   the item is appended to the end (matching Python semantics).
    ///
    /// Returns `Object::None`, matching Python's behavior where `list.insert()` returns None.
    pub fn insert(&mut self, heap: &mut Heap, index: usize, item: Object) -> Object {
        // Increment refcount if item is heap-allocated
        if let Object::Ref(item_id) = &item {
            heap.inc_ref(*item_id);
        }

        // Python's insert() appends if index is out of bounds
        if index >= self.0.len() {
            self.0.push(item);
        } else {
            self.0.insert(index, item);
        }

        Object::None
    }

    /// Add to a stack of ids for for `dec_ref`
    pub fn push_stack_ids(&self, stack: &mut Vec<ObjectId>) {
        // Walk through all items and enqueue any heap-allocated objects
        for obj in &self.0 {
            if let Object::Ref(id) = obj {
                stack.push(*id);
            }
        }
    }

    /// Calls an attribute method on this list (e.g., list.append()).
    ///
    /// This method dispatches to the appropriate list method based on the
    /// attribute name, handling argument validation and conversion.
    ///
    /// # Arguments
    /// * `heap` - Mutable reference to the heap for reference counting
    /// * `attr` - The attribute/method being called
    /// * `args` - Vector of arguments passed to the method
    ///
    /// # Returns
    /// * `Ok(Object)` - The result of the method call (typically None)
    /// * `Err(SimpleException)` - If the attribute doesn't exist or arguments are invalid
    ///
    /// # Supported Methods
    /// * `append(item)` - Appends item to the end of the list
    /// * `insert(index, item)` - Inserts item at the specified index
    pub fn call_attr<'c>(&mut self, heap: &mut Heap, attr: &Attr, args: Vec<Object>) -> RunResult<'c, Object> {
        match attr {
            Attr::Append => {
                if args.len() != 1 {
                    return exc_err_fmt!(
                        ExcType::TypeError;
                        "append() takes exactly one argument ({} given)",
                        args.len()
                    );
                }
                Ok(self.append(heap, args.into_iter().next().unwrap()))
            }
            Attr::Insert => {
                if args.len() != 2 {
                    return exc_err_fmt!(
                        ExcType::TypeError;
                        "insert() expected 2 arguments, got {}",
                        args.len()
                    );
                }
                let mut args_iter = args.into_iter();
                let index = args_iter.next().unwrap().as_int()? as usize;
                let item = args_iter.next().unwrap();
                Ok(self.insert(heap, index, item))
            }
            Attr::Other(_) => {
                exc_err_fmt!(
                    ExcType::AttributeError;
                    "'list' object has no attribute '{}'",
                    attr
                )
            }
        }
    }
}
