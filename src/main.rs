use futures::prelude::*;
use rupnp::Error as RupnpError;
use ssdp_client::{SearchResponse, SearchTarget};
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // UPnPError(UPnPError)
    #[error("error trying to discover devices: {0}")]
    SSDPError(#[from] ssdp_client::Error),
    #[error("error reading response for {0}: {1}")]
    NetworkError(String, hyper::Error),
    #[error("rupnp error for {0}: {1}")]
    RupnpError(String, RupnpError),
    #[error("could not subscribe to events: no local ipv4 interface open")]
    NoLocalInterfaceOpen,
    #[error("An error occurred trying to connect to device: {0}")]
    IO(std::io::Error),
    #[error("invalid url: {0}")]
    InvalidUrl(#[from] rupnp::http::uri::InvalidUri),
    // #[error("invalid utf8: {}")]
    // InvalidUtf8(Utf8Error),
    #[error("{0}")]
    ParseError(&'static str),
    #[error("The control point responded with status code {0}")]
    HttpErrorCode(http::StatusCode),
    // #[error("failed to parse xml: {0}")]
    // XmlError(roxmltree::Error),
    #[error("`{0}` does not contain a `{1}` element or attribute")]
    XmlMissingElement(String, String),
    #[error("Invalid response: {0}")]
    InvalidResponse(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("{0}")]
    JoinErr(#[from] tokio::task::JoinError),
}

use rupnp::Device;

pub async fn discover(
    search_target: &SearchTarget,
    timeout: Duration,
) -> Result<impl Stream<Item = Result<SearchResponse, Error>>, Error> {
    Ok(ssdp_client::search(search_target, timeout, 3, None)
        .await?
        .map_err(Error::SSDPError)
        .map(|res| res))
    //             .map(|res| {
    //                 let res = res?;
    //                 Ok((res.location().parse()?, res.location().to_string()))
    //             })
    //             .and_then(|(u, l)| Device::from_url(u).map_err(|e| Error::RupnpError(l, e)))
    //             .for_each_concurrent(limit, f);
    // let g = FuturesUnordered::from_iter(s.ite);
    // Ok(g)
    // todo!();
}
#[tokio::main]
async fn main() -> Result<(), Error> {
    let search_target = SearchTarget::RootDevice;
    discover(&search_target, Duration::from_secs(3))
        .await?
        .for_each_concurrent(6, |d| {
            future::ready(d)
                .map(|res| {
                    let res = res?;
                    Ok((res.location().parse()?, res.location().to_string()))
                })
                .and_then(|(u, l)| Device::from_url(u).map_err(|e| Error::RupnpError(l, e)))
                .map(|d| {
                    if let Ok(device) = d {
                        // let service = device
                        //     .find_service(&YAMAHA_REMOTE)
                        //     .expect("searched for RenderingControl, got something else");

                        // let args = "<InstanceID>0</InstanceID><Channel>Master</Channel>";
                        // let response = service.action(device.url(), "GetVolume", args).await?;

                        // let volume = response.get("CurrentVolume").unwrap();

                        // println!("'{}' is at volume {}", device.friendly_name(), volume);
                        println!("Found '{}'", device.friendly_name());
                        println!("Services: ");
                        device
                            .services_iter()
                            .for_each(|s| println!(" - '{}'", s.service_id()));
                    } else if let Err(err) = d {
                        eprintln!("{} {0:?}", err)
                    }
                })
        })
        .await;
    // pin_utils::pin_mut!(devices);
    // futures::select! {
    //     _ = discovery => println!("Done!")
    // }
    Ok(())
}
