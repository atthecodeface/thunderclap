//a Modules
mod arg_count;
mod builder;
mod command_set;
mod handler;
mod traits;

pub mod json;

pub use arg_count::ArgCount;
pub use builder::CommandBuilder;
pub use traits::{CommandArgs, CommandArgsValue};

pub(crate) use command_set::CommandSet;
pub(crate) use handler::CommandHandlerSet;
pub(crate) use traits::{ArgFn, ArgResetFn, CommandFn};

pub fn bound<F, V>(v: V, min: Option<V>, max: Option<V>, f: F) -> Result<V, String>
where
    V: PartialOrd,
    F: FnOnce(V, bool) -> String,
{
    if let Some(min) = min {
        if v < min {
            return Err(f(v, false));
        }
    }
    if let Some(max) = max {
        if v > max {
            return Err(f(v, true));
        }
    }
    Ok(v)
}
