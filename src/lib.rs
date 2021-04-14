use std::{
    intrinsics::copy_nonoverlapping,
    marker::PhantomData,
    mem::{align_of, forget, size_of},
};

struct InternalKey {
    offset: KeyIndex,
    size: usize,
}

type KeyIndex = usize;

struct HVec {
    raw: Vec<u8>,
    keys_given_out: Vec<InternalKey>,
}

struct Key<T> {
    offset: KeyIndex,
    _phantom: PhantomData<T>,
}

impl HVec {
    pub fn new() -> Self {
        Self {
            raw: vec![],
            keys_given_out: vec![],
        }
    }

    pub fn push<V>(&mut self, value: V) -> Key<V> {
        // Make sure we have enough space for the allocation + allocation (conversative since we might not need to pad alignment)
        self.raw.reserve(size_of::<V>() + align_of::<V>());

        // now that we know we have room, we can assume that the ptr of the vec does not move

        let ptr: *const V = &value;
        let ptr: *const u8 = ptr as *const u8;

        // References always have to be correctly alligned,
        // and since we construct references to the elements in
        // the Vec, we need to allign the data correctly.
        // We do this by adding alignment padding before
        // the actual value for each insert
        let start_vec_ptr = self.raw.as_mut_ptr();
        let old_len = self.raw.len();
        let end_vec_ptr = unsafe { start_vec_ptr.add(old_len) };
        let alignment_padding = end_vec_ptr.align_offset(align_of::<&V>());
        let size_to_allocate = alignment_padding + size_of::<V>();

        // we want to write to the
        unsafe {
            copy_nonoverlapping(
                ptr,
                end_vec_ptr.offset(alignment_padding as isize),
                size_of::<V>(),
            )
        };
        let new_len = old_len + size_to_allocate;
        unsafe { self.raw.set_len(new_len) };

        let key = Key {
            // the value is stored just after allignment padding
            offset: old_len + alignment_padding,
            _phantom: PhantomData,
        };

        self.keys_given_out.push(InternalKey {
            offset: key.offset,
            size: size_of::<V>(),
        });

        // make sure that the value is not dropped
        // since we will do it when the item is removed from the vec
        forget(value);
        return key;
    }

    pub fn get<'a, V>(&'a self, key: Key<V>) -> &'a V {
        let ptr: *const V = unsafe { self.raw.as_ptr().add(key.offset) as *const V };
        unsafe { ptr.as_ref().unwrap() }
    }
}

impl Drop for HVec {
    fn drop(&mut self) {
        for key in self.keys_given_out {
            // Access the items, and drop
            let ptr: *const V = unsafe { self.raw.as_ptr().add(key.offset) as *const V };
            let v = unsafe { ptr.read() };
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::HVec;

    #[test]
    fn it_works() {
        let mut hvec = HVec::new();
        let v = 1u8;
        let k = hvec.push(v);
        let ret = hvec.get(k);
        assert_eq!(v, *ret);

        let v2 = vec![1];
        let v2_same = v2.clone();
        let k2 = hvec.push(v2);
        assert_eq!(&v2_same, hvec.get(k2));
    }
}
