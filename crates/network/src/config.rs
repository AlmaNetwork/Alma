use anyhow::{Result, anyhow};
use clap::{App, Arg, ArgMatches};

pub fn get_cli_matches() -> ArgMatches {
    App::new("WebRTC Echo Server")
        .version("1.0")
        .author("Your Name")
        .about("A simple WebRTC echo server")
        .arg(
            Arg::with_name("mode")
                .short('m')
                .long("mode")
                .value_name("MODE")
                .help("Sets the mode: 'offer' or 'answer'")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("Sets the port number")
                .default_value("8080")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("remote_address")
                .short('r')
                .long("remote-address")
                .value_name("REMOTE_ADDRESS")
                .help("Sets the remote address (required for offer mode)")
                .takes_value(true),
        )
        .get_matches()
}

pub fn parse_cli_args(matches: &ArgMatches) -> Result<(String, String, String)> {
    let mode = matches.value_of("mode").unwrap().to_string();
    let port = matches.value_of("port").unwrap().to_string();
    let remote_address = matches.value_of("remote_address").map(|s| s.to_string()).unwrap_or_default();

    if mode == "offer" && remote_address.is_empty() {
        return Err(anyhow!("Remote address is required for offer mode"));
    }

    Ok((mode, port, remote_address))
}