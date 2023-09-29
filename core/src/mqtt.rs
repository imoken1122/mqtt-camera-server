use async_std;
use futures::executor::block_on;
use futures::StreamExt;
use paho_mqtt as mqtt;
use std::time::Duration;
use std::{env, process};

pub fn publisher() {
    let host = env::args()
        .nth(1)
        .unwrap_or_else(|| "localhost:1883".to_string());
    println!("Connecting to the MQTT server at '{}'", host);

    let cli = mqtt::AsyncClient::new(host).unwrap_or_else(|err| {
        print!("Error creating the client: {:?}", err);
        process::exit(1);
    });

    if let Err(err) = block_on(async {
        cli.connect(None).await?;

        println!("Publishing to the topic 'test'...");
        let msg = mqtt::Message::new("test", "Hello world!", mqtt::QOS_1);
        cli.publish(msg).await?;

        println!("Disconnecting...");
        cli.disconnect(None).await?;
        Ok::<(), mqtt::Error>(())
    }) {
        eprint!("Error: {}", err);
    }
}

const TOPICS: &[&str] = &["test", "hello"];
const QOS: &[i32] = &[1, 1];

pub fn subscriber() {
    let host = env::args()
        .nth(1)
        .unwrap_or_else(|| "tcp://localhost:1883".to_string());

    println!("Connecting to the MQTT server at '{}'...", host);

    let create_opts = mqtt::CreateOptionsBuilder::new_v3()
        .server_uri(host)
        .client_id("async-subscriber")
        .finalize();

    let mut cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|e| {
        println!("Error creating the client: {:?}", e);
        process::exit(1);
    });
    if let Err(err) = block_on(async {
        let mut strm = cli.get_stream(25);

        let lwt = mqtt::Message::new("test/lwt", "Async subscriber lost connection", mqtt::QOS_1);
        let conn_opts = mqtt::ConnectOptionsBuilder::new_v3()
            .keep_alive_interval(Duration::from_secs(30))
            .clean_session(false)
            .will_message(lwt)
            .finalize();

        cli.connect(conn_opts).await?;

        println!("Subscribing to topics: ");
        cli.subscribe_many(TOPICS, QOS).await?;
        println!("waiting for messages...");

        while let Some(msg_opt) = strm.next().await {
            if let Some(msg) = msg_opt {
                println!("{:?}", msg);
            } else {
                println!("Lost connection attempting reconnect");
                while let Err(err) = cli.reconnect().await {
                    println!("Error reconnecting: {:?}", err);
                    async_std::task::sleep(Duration::from_secs(1));
                }
            }
        }
        Ok::<(), mqtt::Error>(())
    }) {
        eprint!("Error: {}", err);
    }
}
