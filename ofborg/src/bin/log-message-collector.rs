extern crate ofborg;
extern crate amqp;
extern crate env_logger;

use std::env;
use std::path::PathBuf;

use ofborg::config;
use ofborg::worker;
use ofborg::tasks;
use ofborg::easyamqp;
use ofborg::easyamqp::{Exchange, TypedWrappers};


fn main() {
    let cfg = config::load(env::args().nth(1).unwrap().as_ref());
    ofborg::setup_log();

    let mut session = easyamqp::session_from_config(&cfg.rabbitmq).unwrap();
    println!("Connected to rabbitmq");

    let mut channel = session.open_channel(1).unwrap();

    channel
        .declare_exchange(easyamqp::ExchangeConfig {
            exchange: Exchange("logs"),
            exchange_type: easyamqp::ExchangeType::Topic,
            passive: false,
            durable: true,
            auto_delete: false,
            no_wait: false,
            internal: false,
            arguments: None,
        })
        .unwrap();

    let queue_name = channel
        .declare_queue(easyamqp::QueueConfig {
            queue: "",
            passive: false,
            durable: false,
            exclusive: true,
            auto_delete: true,
            no_wait: false,
            arguments: None,
        })
        .unwrap()
        .queue;

    channel
        .bind_queue(easyamqp::BindQueueConfig {
            queue: &queue_name,
            exchange: "logs",
            routing_key: Some("*.*"),
            no_wait: false,
            arguments: None,
        })
        .unwrap();

    channel
        .consume(
            worker::new(tasks::log_message_collector::LogMessageCollector::new(
                PathBuf::from(cfg.log_storage.clone().unwrap().path),
                100,
            )),
            easyamqp::ConsumeConfig {
                queue: &queue_name,
                consumer_tag: &format!("{}-log-collector", cfg.whoami()),
                no_local: false,
                no_ack: false,
                no_wait: false,
                exclusive: false,
                arguments: None,
            },
        )
        .unwrap();


    channel.start_consuming();

    println!("Finished consuming?");

    channel.close(200, "Bye").unwrap();
    println!("Closed the channel");
    session.close(200, "Good Bye");
    println!("Closed the session... EOF");

}
