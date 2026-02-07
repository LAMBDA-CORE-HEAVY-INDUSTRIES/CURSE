use std::io::{self, Read};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use probe_rs::rtt::Rtt;
use probe_rs::{Permissions, Session, SessionConfig};

fn main() -> Result<()> {
    let mut chip = String::from("STM32F411RE");
    let mut down_channel: usize = 0;
    let mut speed: Option<u32> = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--chip" => {
                chip = args
                    .next()
                    .context("missing value for --chip")?;
            }
            "--down" => {
                down_channel = args
                    .next()
                    .context("missing value for --down")?
                    .parse()
                    .context("invalid --down value")?;
            }
            "--speed" => {
                speed = Some(
                    args.next()
                        .context("missing value for --speed")?
                        .parse()
                        .context("invalid --speed value")?,
                );
            }
            "--help" | "-h" => {
                println!(
                    "Usage: rtt-raw [--chip <name>] [--down <n>] [--speed <khz>]\n\
                     Defaults: --chip STM32F411RE --down 0\n\
                     Press Ctrl-C to exit."
                );
                return Ok(());
            }
            other => bail!("unknown arg: {other}"),
        }
    }

    let mut cfg = SessionConfig::default();
    cfg.permissions = Permissions::default();
    cfg.speed = speed;

    let mut session = Session::auto_attach(&chip, cfg).context("attach session")?;
    let mut core = session.core(0).context("attach core 0")?;

    let start = Instant::now();
    let mut rtt = loop {
        match Rtt::attach(&mut core) {
            Ok(rtt) => break rtt,
            Err(err) => {
                if start.elapsed() > Duration::from_secs(5) {
                    return Err(err).context("RTT control block not found");
                }
                thread::sleep(Duration::from_millis(100));
            }
        }
    };

    let down = rtt.down_channel(down_channel).context("down channel not found")?;
    enable_raw_mode().context("enable raw mode")?;
    let mut stdin = io::stdin();
    let mut buf = [0u8; 1];
    loop {
        if let Err(err) = stdin.read_exact(&mut buf) {
            disable_raw_mode().ok();
            return Err(err).context("read stdin");
        }
        if buf[0] == 3 {
            break;
        }
        let _ = down.write(&mut core, &buf)?;
    }
    disable_raw_mode().ok();
    Ok(())
}
