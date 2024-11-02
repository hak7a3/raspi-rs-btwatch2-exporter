use std::{
    error::Error,
    net::SocketAddr,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use bluer::{AdapterEvent, Address, DeviceEvent, DeviceProperty};
use clap::Parser;
use tokio_stream::{StreamExt, StreamMap};

mod btwatch2;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("0.0.0.0:9000"))]
    server: String,
    #[arg(short, long)]
    device: String,
}

#[tokio::main]
#[allow(unreachable_code)]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args = Args::parse();

    log::info!("start rt-btwatch2 exporter");

    init_prometheus(&args.server)?;
    log::info!("start prometheus server at {:}", args.server);

    let voltage = metrics::gauge!("voltage_volt");
    let current = metrics::gauge!("current_ampere");
    let power = metrics::gauge!("power_watt");
    let last_measured = metrics::gauge!("last_measured_timestamp_ms");

    let target = Address::from_str(&args.device)?;
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    let mut device_events = adapter.discover_devices().await?;
    let mut change_events = StreamMap::new();
    loop {
        tokio::select! {
            Some(event) = device_events.next() => {
                if let AdapterEvent::DeviceAdded(addr) = event {
                    if addr == target {
                        log::info!("target device is found");
                        let device = adapter.device(addr)?;
                        change_events.insert(addr, device.events().await?);
                    } else {
                        log::trace!("other add {:?}", event);
                    }
                }
            }

            Some((_, DeviceEvent::PropertyChanged(property))) = change_events.next() => {
                if let DeviceProperty::ManufacturerData(hash_map) = property {
                    if let Some(value) = hash_map.get(&0x0b60) {
                        let timestamp =
                        SystemTime::now().duration_since(UNIX_EPOCH)
                            .inspect_err(|e| log::warn!("failed to get current time: {:?}", e))
                            .map(|d| d.as_millis() as f64)
                            .unwrap_or_default();

                        let parsed = btwatch2::parse_manufacturer_data(value);
                        voltage.set(parsed.voltage);
                        current.set(parsed.current);
                        power.set(parsed.power);
                        last_measured.set(timestamp);
                    }
                }
            }
        }
    }
    return Ok(());
}

fn init_prometheus(addr: &str) -> Result<(), Box<dyn Error>> {
    let socket = SocketAddr::from_str(addr)?;

    let builder = metrics_exporter_prometheus::PrometheusBuilder::new();
    builder.with_http_listener(socket).install()?;

    return Ok(());
}
