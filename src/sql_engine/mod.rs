mod sql_codegen;
mod sql_error;
mod sql_lexer;
mod sql_optimizer;
mod sql_parser;
mod sql_tests;

/// This function turns a string of SQL into optimized SQL Bytecode.
/// Currently unimplemented.
/// Honest function.
pub fn compile_sql(code: String) -> (Vec<SQL_Bytecode>, SQL_Constants) {
    unimplemented!()
}

/// This is bytecode meant to run on a virtual machine in the storage engine.
/// It is stack-based, and always comes with a set of constants.
/// For example, when a user writes code with a string or number literal in it,
/// that is put in the set of constants, and is reffered to in the bytecode by its
/// index. There are different sets of constants for each type.
pub enum SQL_Bytecode {}

/// This is the constants of the bytecode, and they are always meant to go together in execution.
/// The engine allows you to use arbitrary bytes as keys or values as well as defined types.``
pub struct SQL_Constants {
    pub table_id_constants: Vec<String>,
    pub integer_constants: Vec<i32>,
    // string_constants: Vec<String>,
    pub arbitrary_bytes_constants: Vec<(u8, Vec<u8>)>,
}

/// This is the executable content able to be used by the SQL VM in the storage engine.
/// It contains both the actual bytecode and constants.
pub struct SQL_Executable {
    pub bytecode: Vec<SQL_Bytecode>,
    pub constants: SQL_Constants,
}
