mod query_input;
mod sql_engine;
mod storage_engine;

fn main() {
    unimplemented!()
}

/// This function will find where there is a db folder already. It will first check
/// the path in the home directory, then the one in (undecided location near root directory,
/// probably in /var somewhere.).
/// If it can't find anything, returns None.
fn find_db_directory_path() -> Option<String> {
    unimplemented!()
}

/// This function will create a new db directory in the given path, returning None if it cannot
/// do so. (use an actual error value later)
fn create_db_directory(path: String) -> Option<()> {
    unimplemented!()
}
