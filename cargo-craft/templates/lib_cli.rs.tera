pub trait ParserDispatcher<E: std::error::Error>: clap::Parser {
    fn dispatch(&self) -> Result<(), E>;
    fn dispatch_cargo(&self) -> Result<(), E> {
        Ok(self.dispatch()?)
    }
    fn main() -> Result<(), E> {
        let (args, is_cargo) = Self::args();
        if is_cargo {
            Self::dispatch_cargo(&Self::parse_from(&args))?;
        } else {
            Self::dispatch(&Self::parse_from(&args))?;
        }
        Ok(())
    }
    fn args() -> (Vec<String>, bool) {
        let args = iocore::env::args();
        let execname = iocore::Path::new(&args[0]).name();
        let is_cargo = execname.starts_with("cargo");
        let args = if is_cargo { args[1..].to_vec() } else { args.to_vec() };
        (args, is_cargo)
    }
}


pub trait SubcommandDispatcher<E: std::error::Error>: clap::Subcommand {
    fn dispatch(&self) -> Result<(), E>;
}

pub trait ArgsDispatcher<E: std::error::Error>: clap::Args {
    fn dispatch(&self) -> Result<(), E>;
}
