/// The configuration parameters for the application.
///
/// These can either be passed on the command line, or pulled from environment variables.
///
/// For development convenience, these can also be read from a `.env` file in the working
/// directory where the application is started.
///
/// See `.env.sample` in the repository root for details.
#[derive(clap::Parser)]
pub struct Config {
    /// The connection URL for the SQLite database this application should use.
    #[clap(long, env)]
    pub database_url: String,
}
