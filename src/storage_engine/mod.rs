mod compaction;
mod concurrency_control;
mod db_errors;
mod db_tests;
mod disk_io;
mod page_cache;
mod parallel_work_runner;
mod recovery_manager;
mod sql_executor;
mod write_ahead_logger;

/// Will start the actual execution of the DBMS. It will never stop unless it cannot find the directory
/// given as an input. It starts by booting up the DBMS with checking version for binary formats,
/// using the recovery manager to recover lost data stated in the WAL, and generally figure everything out.
/// After that, it's ready to go, assigns all the threads needed, and is off to the races!
/// Dishonest function.
pub fn run_dbms(directory_path: String) -> Option<()> {
    unimplemented!()
}
