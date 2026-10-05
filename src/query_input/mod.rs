mod stdin_queries;
mod network_queries;


/// This is a dishonest function that will block until it recieves a query from stdin or the network,
/// upon which it returns said query. Meant to be ran in a loop on a single thread, and pushed into
/// a stack for worker threads to work on.
pub fn get_next_query() -> DB_Query {
    unimplemented!()
}

/// This can either be a query that needs compiling, or a query that can be directly executed.
/// Either way, it is parsed by a worker thread.
pub enum DB_Query {
    SQL_Query(String),
    Direct_Query(Vec<u8>),
}
