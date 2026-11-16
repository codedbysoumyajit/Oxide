pub mod client;
pub mod protocol;

pub use client::DebugSession;
pub use protocol::{
    Breakpoint, DebugState, Scope, SourceBreakpoint, StackFrame, Variable,
};
