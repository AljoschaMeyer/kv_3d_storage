// use core::future::Future;

// /// A persistent storage backend that maps bytestrings keys to values of some type `V`, and allows for efficient access based on the lexicographic ordering of the keys.
// pub trait BackEnd<V> {
//     /// Type of errors that can occur when interacting with the backend.
//     type Error;

//     /// Insert a kv pair. Returns the old value for that key, if there was any.
//     ///
//     /// This need not be persisted to disk immediately, persistence may be delayed until [`flush`](Self::flush) is called. All subsequent method calls must incorporat the insertion though, even if it has not been persisted yet.
//     fn insert(
//         &mut self,
//         key: &[u8],
//         value: V,
//     ) -> impl Future<Output = Result<Option<V>, Self::Error>>;

//     /// Delete a kv pair. Returns the old value for that key, if there was any.
//     ///
//     /// This need not be persisted to disk immediately, persistence may be delayed until [`flush`](Self::flush) is called. All subsequent method calls must incorporat the deletion though, even if it has not been persisted yet.
//     fn delete(&mut self, key: &[u8]) -> impl Future<Output = Result<Option<V>, Self::Error>>;

//     /// Commit all mutations that have been performed so far to disk. When the Future is done, the changes are guaranteed to be persisted.
//     fn flush(&mut self) -> impl Future<Output = Result<(), Self::Error>>;

//     /// Get the value associated with the given key, if there is any.
//     fn get(&self, key: &[u8]) -> impl Future<Output = Result<Option<V>, Self::Error>>;

//     /// Get the greatest kv pair whose key is less than or equal to the given key, if there is any.
//     fn find_lte(&self, key: &[u8])
//         -> impl Future<Output = Result<Option<(&[u8], V)>, Self::Error>>;

//     /// Get the least kv pair whose key is greater than or equal to the given key, if there is any.
//     fn find_gte(&self, key: &[u8])
//         -> impl Future<Output = Result<Option<(&[u8], V)>, Self::Error>>;
// }

// // TODO batch/transaction
