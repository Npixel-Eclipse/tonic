use futures::TryStreamExt;
use route_guide::{Point, RouteNote};
use std::time::{Duration, Instant};
use tokio::timer::Interval;
use tonic::{transport::Channel, Request};

mod route_guide {
    include!(concat!(env!("OUT_DIR"), "/routeguide.rs"));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let origin = http::Uri::from_static("http://[::1]:10000");

    let svc = Channel::builder().build(origin)?;
    let mut client = route_guide::RouteGuideClient::new(svc);

    let start = Instant::now();

    let response = client
        .get_feature(Request::new(Point {
            latitude: 409146138,
            longitude: -746188906,
        }))
        .await?;

    println!("FEATURE = {:?}", response);

    let outbound = async_stream::try_stream! {
        let mut interval =  Interval::new_interval(Duration::from_secs(1));

        while let Some(time) = interval.next().await {
            let elapsed = time.duration_since(start);
            let note = RouteNote {
                location: Some(Point {
                    latitude: 409146138 + elapsed.as_secs() as i32,
                    longitude: -746188906,
                }),
                message: format!("at {:?}", elapsed),
            };

            yield note;
        }
    };

    let request = Request::new(outbound);

    let response = client.route_chat(request).await?;

    let mut inbound = response.into_inner();

    while let Some(note) = inbound.try_next().await? {
        println!("NOTE = {:?}", note);
    }

    Ok(())
}