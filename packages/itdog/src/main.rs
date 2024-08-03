use cote::prelude::*;
use itdog::ItdogClient;
use itdog::DEFAULT_KEY;
use prettytable::Row;
use prettytable::Table;
use tracing::level_filters::LevelFilter;

#[derive(Debug, Cote)]
#[cote(aborthelp, width = 100)]
pub struct Httping {
    /// Set the key of request
    #[arg(alias = "-k", value = DEFAULT_KEY)]
    key: String,

    /// The target url, for example: www.baidu.com
    #[pos(force = true)]
    host: String,

    /// Enable debug mode
    #[arg(alias = "-d")]
    debug: bool,

    /// Enable verbose mode
    #[arg(alias = "-v")]
    verbose: bool,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut subscriber = tracing_subscriber::fmt::fmt();
    let Httping {
        key,
        host,
        debug,
        verbose,
    } = Httping::parse_env()?;

    if debug {
        subscriber = subscriber.with_max_level(LevelFilter::DEBUG);
        if verbose {
            subscriber = subscriber.with_max_level(LevelFilter::TRACE);
        }
    }
    subscriber
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    rustls::crypto::ring::default_provider()
        .install_default()
        .unwrap();

    let (_, cancell) = tokio::sync::oneshot::channel();
    let (send, mut recv) = tokio::sync::mpsc::channel(128);

    tokio::spawn(async move {
        let mut itdog = ItdogClient::new(&key, &host, cancell, send);
        itdog.query().await.unwrap();
    });

    let mut messages = vec![];

    while let Some(req) = recv.recv().await {
        messages.push(req);
    }

    let mut table = Table::new();

    table.add_row(Row::from_iter(itdog::Message::construct_header()));
    if !messages.is_empty() {
        messages
            .iter()
            .map(|msg| msg.construct_row())
            .for_each(|v| {
                table.add_row(Row::from_iter(v));
            });
        table.add_row(Row::from_iter(itdog::Message::construct_header()));
        table.printstd();
    }
    Ok(())
}
