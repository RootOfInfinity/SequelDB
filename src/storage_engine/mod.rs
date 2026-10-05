mod compaction.rs;
mod concurrency_control.rs;
mod disk_io.rs;
mod page_cache.rs;
mod parallel_work_runner.rs;
mod recovery_manager.rs;
mod sql_executor.rs;
mod write_ahead_logger.rs;
mod db_errors.rs;
mod db_tests.rs;

/// Will start the actual execution of the DBMS. It will never stop unless it cannot find the directory
/// given as an input. It starts by booting up the DBMS with checking version for binary formats,
/// using the recovery manager to recover lost data stated in the WAL, and generally figure everything out.
/// After that, it's ready to go, assigns all the threads needed, and is off to the races!
/// Dishonest function.
pub fn run_dbms(directory_path: String) -> Option<!> {
    unimplemented!()
}

