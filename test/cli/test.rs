use clap::{Parser, Subcommand, Args, ValueEnum};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[arg(long, default_value = "info", env = "GADGETS_LOG_LEVEL")]
    log_level: LogLevel,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Sync(SyncOpt),
    L(LOpt),
    O(OOpt),
    P(POpt),
    F(FOpt),
    X(XOpt),
    Rep(RepOpt),
    Rm(RmOpt),
    File(FileOpt),
    Argv(ArgvOpt),
    K(KOpt),
    W(WOpt),
    Col(ColOpt),
    Std(StdOpt),
    Sh(ShOpt),
    Mv(MvOpt),
    Enu(EnuOpt),
    Http(HttpOpt),
    Env(EnvOpt),
    Unhex(SechzehnOpt),
    E(EOpt),
    C(ClipboardOpt),
    Clipboard(ClipboardOpt),
    Conf(ConfOpt),
    Enumerate(EnumerateOpt),
    Tls(TlsOpt),
    Vc(VersionControlOpt),
    Path(PathOpt),
    Sort(SortOpt),
    Cap(CapOpt),
    Wifi(WifiOpt),
    A(AOpt),
    D(DOpt),
    Daemon(DaemonOpt),
    Totp(TotpOpt),
}

fn main() -> Result<()> {
    Path::from(format!(
        "/tmp/{}.{}",
        Utc::now().format("%Y%m%d_%H%M%S").to_string(),
        ::std::process::id()
    ))
    .write(format!("{}", ::std::process::id()).as_bytes())?;
    let args = Cli::parse();
    let log_level = args.log_level.clone();
    gadgets::cli::init(log_level);
    match match args.command {
        Command::Sync(mut op) => op.dispatch(),
        Command::L(mut op) => op.dispatch(),
        Command::O(mut op) => op.dispatch(),
        Command::P(mut op) => op.dispatch(),
        Command::F(mut op) => op.dispatch(),
        Command::X(mut op) => op.dispatch(),
        Command::Rep(mut op) => op.dispatch(),
        Command::Rm(mut op) => op.dispatch(),
        Command::File(mut op) => op.dispatch(),
        Command::Argv(mut op) => op.dispatch(),
        Command::K(mut op) => op.dispatch(),
        Command::W(mut op) => op.dispatch(),
        Command::Col(mut op) => op.dispatch(),
        Command::Std(mut op) => op.dispatch(),
        Command::Sh(mut op) => op.dispatch(),
        Command::Mv(mut op) => op.dispatch(),
        Command::Enu(mut op) => op.dispatch(),
        Command::Http(mut op) => op.dispatch(),
        Command::Unhex(mut op) => op.dispatch(),
        Command::E(mut op) => op.dispatch(),
        Command::Conf(mut op) => op.dispatch(),
        Command::Env(mut op) => op.dispatch(),
        Command::Enumerate(mut op) => op.dispatch(),
        Command::C(mut op) | Command::Clipboard(mut op) => op.dispatch(),
        Command::Tls(mut op) => op.dispatch(),
        Command::Vc(mut op) => op.dispatch(),
        Command::Path(mut op) => op.dispatch(),
        Command::Sort(mut op) => op.dispatch(),
        Command::Cap(mut op) => op.dispatch(),
        Command::A(mut op) => op.dispatch(),
        Command::D(mut op) => op.dispatch(),
        Command::Wifi(mut op) => op.dispatch(),
        Command::Daemon(mut op) => op.dispatch(),
        Command::Totp(mut op) => op.dispatch(),
    } {
        Ok(_) => Ok(()),
        Err(e) => {
            match e {
                Error::NonFitToPrint(_) => {},
                _ => {
                    eprintln!("{}", gadgets::ansi::e(e.to_string(), 160));
                },
            }
            ::std::process::exit(0xF >> 1);
        },
    }
}
