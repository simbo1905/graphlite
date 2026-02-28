/// Python 3.13+ emitter — generates standalone validation modules.
mod context;
mod emit;
mod writer;

pub use emit::emit;
