// use feeders::Feeder;
// use futures_util::stream::StreamExt;
// use rdkafka::{
//     ClientConfig, Message,
//     consumer::{Consumer, StreamConsumer},
//     producer::FutureRecord,
// };
// use std::time::Duration;
// use testcontainers::runners::AsyncRunner;
// use testcontainers_modules::kafka::confluent::{self, Kafka};

// #[tokio::test]
// async fn feeder_producer_should_work() -> Result<(), Box<dyn std::error::Error + 'static>> {
//     let kafka_container = Kafka::default()
//         .start()
//         .await
//         .expect("Failed to start Kafka container");

//     let brokers = format!(
//         "127.0.0.1:{}",
//         kafka_container
//             .get_host_port_ipv4(confluent::KAFKA_PORT)
//             .await?
//     );
//     let feeder = Feeder::create(&brokers).expect("should create feeder");

//     let consumer = ClientConfig::new()
//         .set("group.id", "testcontainer-rs")
//         .set("bootstrap.servers", &brokers)
//         .set("session.timeout.ms", "6000")
//         .set("enable.auto.commit", "false")
//         .set("auto.offset.reset", "earliest")
//         .create::<StreamConsumer>()
//         .expect("Failed to create Kafka StreamConsumer");

//     let topic = "test-topic";

//     let number_of_messages_to_produce = 5_usize;
//     let expected: Vec<String> = (0..number_of_messages_to_produce)
//         .map(|i| format!("Message {i}"))
//         .collect();

//     for (i, message) in expected.iter().enumerate() {
//         feeder
//             .producer
//             .send(
//                 FutureRecord::to(topic)
//                     .payload(message)
//                     .key(&format!("Key {i}")),
//                 Duration::from_secs(0),
//             )
//             .await
//             .unwrap();
//     }

//     consumer
//         .subscribe(&[topic])
//         .expect("Failed to subscribe to a topic");

//     let mut message_stream = consumer.stream();
//     for produced in expected {
//         let borrowed_message = tokio::time::timeout(Duration::from_secs(10), message_stream.next())
//             .await
//             .unwrap()
//             .unwrap();

//         assert_eq!(
//             produced,
//             borrowed_message
//                 .unwrap()
//                 .payload_view::<str>()
//                 .unwrap()
//                 .unwrap()
//         );
//     }

//     Ok(())
// }
