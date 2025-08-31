//a Imports
use std::collections::HashMap;
use std::rc::Rc;

use clap::{value_parser, Arg, ArgAction, Command};

use crate::{ArgCount, ArgFn, ArgResetFn, CommandArgs, CommandFn, CommandHandlerSet, CommandSet};

//a Useful functions
//mi into_box
/// This function si required to make Rust happy
///
/// Box::new will not map from a Fn to a dyn
fn into_box<'lt, C: CommandArgs, F: 'lt + CommandFn<C>>(x: F) -> Box<dyn 'lt + CommandFn<C>> {
    Box::new(x)
}

//a CommandBuilder
//tp CommandBuilder
pub struct CommandBuilder<C: CommandArgs> {
    command: Command,
    handler_set: CommandHandlerSet<C>,
    sub_cmds: HashMap<String, CommandBuilder<C>>,
}

//ip Default for CommandBuilder
impl<C: CommandArgs> std::default::Default for CommandBuilder<C> {
    fn default() -> Self {
        let command = Command::default();
        let handler_set = CommandHandlerSet::default();
        let sub_cmds = HashMap::default();
        Self {
            command,
            handler_set,
            sub_cmds,
        }
    }
}

//ip CommandBuilder

impl<C: CommandArgs> CommandBuilder<C> {
    //ci create_with_opt_handler
    pub fn create_with_opt_handler(
        command: Command,
        handler: Option<Box<dyn CommandFn<C> + 'static>>,
    ) -> Self {
        let handler_set = CommandHandlerSet::new(handler);
        let sub_cmds = HashMap::default();
        Self {
            command,
            handler_set,
            sub_cmds,
        }
    }

    //cp new
    pub fn new(mut command: Command) -> Self {
        command = command.subcommand_required(true);
        Self::create_with_opt_handler(command, None)
    }

    //cp with_handler
    pub fn with_handler<F>(command: Command, handler: F) -> Self
    where
        F: CommandFn<C> + 'static,
    {
        let handler = into_box(handler);
        Self::create_with_opt_handler(command, Some(handler))
    }

    //mp set_arg_reset
    pub fn set_arg_reset(&mut self, handler: Box<dyn ArgResetFn<C>>) -> &mut Self {
        self.handler_set.set_arg_reset(handler);
        self
    }

    //mp add_arg
    pub fn add_arg(&mut self, arg: Arg, handler: Box<dyn ArgFn<C>>) -> &mut Self {
        let name = arg.get_id().as_str().into();
        self.command = std::mem::take(&mut self.command).arg(arg);
        self.handler_set.add_arg(name, handler);
        self
    }

    //mp add_subcommand
    pub fn add_subcommand(&mut self, subcommand: Self) -> &mut Self {
        self.sub_cmds
            .insert(subcommand.command.get_name().into(), subcommand);
        self
    }

    //mp build_subcommand
    pub fn build_subcommand(&mut self, subcommand: &mut Self) -> &mut Self {
        self.add_subcommand(std::mem::take(subcommand));
        self
    }

    //mc take
    pub(crate) fn take(self) -> (Command, CommandHandlerSet<C>) {
        let mut command = self.command;
        let mut handler_set = self.handler_set;
        for (k, sc) in self.sub_cmds.into_iter() {
            let (sc, schs) = sc.take();
            handler_set.add_subcommand(k, schs);
            command = command.subcommand(sc);
        }
        (command, handler_set)
    }

    //mp main
    /// Convert the builder into an actual [CommandSet] to be used by 'main'
    pub fn main(self, allow_batch: bool, allow_interactive: bool) -> CommandSet<C> {
        CommandSet::main(self, allow_batch, allow_interactive)
    }

    //mp add_arg_value
    pub fn add_arg_value<F, I>(
        &mut self,
        tag: &'static str,
        short: Option<char>,
        help: &'static str,
        count: I,
        default_value: Option<&'static str>,
        set: F,
    ) where
        F: Fn(&mut C, &Rc<C::Value>) -> Result<(), C::Error> + 'static,
        I: Into<ArgCount>,
    {
        let count = count.into();
        let uses_tag = count.uses_tag();
        let required = count.required();
        let action = count.action();
        let num_args = count.num_args();
        let mut arg = Arg::new(tag)
            .help(help)
            .value_parser(value_parser!(String))
            .required(required)
            .action(action);
        if let Some(num_args) = num_args {
            arg = arg.num_args(num_args);
        }
        if uses_tag {
            arg = arg.long(tag);
        }
        if let Some(short) = short {
            arg = arg.short(short);
        }
        if let Some(default_value) = default_value {
            arg = arg.default_value(default_value);
        }
        self.add_arg(
            arg,
            Box::new(move |cmd_set, args, matches| {
                for v in matches.get_many::<String>(tag).unwrap() {
                    let (_rest, opt_value) = cmd_set.substitute_var(args, v)?;
                    let value = {
                        if let Some(value) = opt_value {
                            value
                        } else {
                            Rc::new(CommandSet::<C>::str_as_value(v)?)
                        }
                    };
                    set(args, &value)?
                }
                Ok(())
            }),
        );
    }

    //mp add_flag
    pub fn add_flag<F>(
        &mut self,
        tag: &'static str,
        short: Option<char>,
        help: &'static str,
        set: F,
    ) where
        F: Fn(&mut C, bool) -> Result<(), C::Error> + 'static,
    {
        let mut arg = Arg::new(tag)
            .long(tag)
            .help(help)
            .action(ArgAction::SetTrue);
        if let Some(short) = short {
            arg = arg.short(short);
        }
        self.add_arg(
            arg,
            Box::new(move |_command_set, args, matches| {
                set(args, *matches.get_one::<bool>(tag).unwrap())
            }),
        );
    }
}

//ap add_arg
macro_rules! add_arg {
    ($t:ty, $tag:expr, $help:expr, $short:expr, $count:expr, $default_value:expr ) => {{
        let uses_tag = $count.uses_tag();
        let required = $count.required();
        let action = $count.action();
        let num_args = $count.num_args();
        let mut arg = Arg::new($tag)
            .help($help)
            .value_parser(value_parser!($t))
            .required(required)
            .action(action);
        if let Some(num_args) = num_args {
            arg = arg.num_args(num_args);
        }
        if uses_tag {
            arg = arg.long($tag);
        }
        if let Some(short) = $short {
            arg = arg.short(short);
        }
        if let Some(default_value) = $default_value {
            arg = arg.default_value(default_value);
        }
        arg
    }};
    ($m:ident, $t: ty, ref $ft:ty ) => {
        /// Add an argument
        ///
        /// 'tag' is how the argument is referred to in clap, and must be unique
        ///
        /// 'short' is the '-X' option used to specify the argument;
        ///    use None for positional arguments, or for nonpositional
        ///    arguments that do not need a short setting.
        ///
        /// 'count' indicates the number of arguments required, and if
        ///    the argument is positional or requires a '--X' option
        ///
        /// 'default_value' is a *string* which is used if the user
        ///    does not provide an option; this will be parsed
        ///    appropriately as if provided by the user.
        ///
        /// 'set' is a callback invoked for every occurrence of the
        ///    argument; if the count indicates a *single* argument at
        ///    most, then it will be invoked at most once
        impl<C: CommandArgs> CommandBuilder<C> {
            pub fn $m<F, I>(
                &mut self,
                tag: &'static str,
                short: Option<char>,
                help: &'static str,
                count: I,
                default_value: Option<&'static str>,
                set: F,
            ) where
                F: Fn(&mut C, &$ft) -> Result<(), C::Error> + 'static,
                I: Into<ArgCount>,
            {
                let count = count.into();
                let arg = add_arg!($t, tag, help, short, count, default_value);

                self.add_arg(
                    arg,
                    Box::new(move |_command_set, args, matches| {
                        for v in matches.get_many::<$t>(tag).unwrap() {
                            set(args, &*v)?
                        }
                        Ok(())
                    }),
                );
            }
        }
    };
    ($m:ident, $t: ty, $ft:ty ) => {
        impl<C: CommandArgs> CommandBuilder<C> {
            pub fn $m<F, I>(
                &mut self,
                tag: &'static str,
                short: Option<char>,
                help: &'static str,
                count: I,
                default_value: Option<&'static str>,
                set: F,
            ) where
                F: Fn(&mut C, $ft) -> Result<(), C::Error> + 'static,
                I: Into<ArgCount>,
            {
                let count = count.into();
                let arg = add_arg!($t, tag, help, short, count, default_value);

                self.add_arg(
                    arg,
                    Box::new(move |_command_set, args, matches| {
                        for v in matches.get_many::<$t>(tag).unwrap() {
                            set(args, *v)?
                        }
                        Ok(())
                    }),
                );
            }
        }
    };
    ($m:ident, $t: ty) => {
        add_arg!($m, $t, $t);
    };
}

add_arg!(add_arg_string, String, ref str);

add_arg!(add_arg_isize, isize);
add_arg!(add_arg_i128, i128);
add_arg!(add_arg_i64, i64);
add_arg!(add_arg_i32, i32);
add_arg!(add_arg_i16, i16);
add_arg!(add_arg_i8, i8);

add_arg!(add_arg_usize, usize);
add_arg!(add_arg_u128, u128);
add_arg!(add_arg_u64, u64);
add_arg!(add_arg_u32, u32);
add_arg!(add_arg_u16, u16);
add_arg!(add_arg_u8, u8);

add_arg!(add_arg_f64, f64);
add_arg!(add_arg_f32, f32);
