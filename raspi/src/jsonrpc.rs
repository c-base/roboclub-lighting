use std::{sync::Arc, thread, time};

use jsonrpc_core::*;
use jsonrpc_pubsub::{PubSubHandler, Session, Subscriber, SubscriptionId};
use jsonrpc_ws_server::{RequestContext, ServerBuilder};

pub fn start() {
	let mut io = PubSubHandler::<Arc<Session>>::new(MetaIoHandler::default());
	io.add_sync_method("say_hello", |_params: Params| {
		Ok(Value::String("hello".to_string()))
	});

	// io.add_subscription(
	// 	"hello",
	// 	(
	// 		"subscribe_hello",
	// 		|params: Params, _, subscriber: Subscriber| {
	// 			if params != Params::None {
	// 				subscriber
	// 					.reject(Error {
	// 						code:    ErrorCode::ParseError,
	// 						message: "Invalid parameters. Subscription rejected.".into(),
	// 						data:    None,
	// 					})
	// 					.unwrap();
	// 				return;
	// 			}
	//
	// 			thread::spawn(move || {
	// 				let sink = subscriber.assign_id(SubscriptionId::Number(5)).unwrap();
	// 				// or subscriber.reject(Error {} );
	// 				// or drop(subscriber)
	//
	// 				loop {
	// 					thread::sleep(time::Duration::from_millis(100));
	// 					match sink.notify(Params::Array(vec![Value::Number(10.into())])) {
	// 						Ok(_) => {}
	// 						Err(_) => {
	// 							println!("Subscription has ended, finishing.");
	// 							break;
	// 						}
	// 					}
	// 				}
	// 			});
	// 		},
	// 	),
	// 	("remove_hello", |_id: SubscriptionId, _| {
	// 		println!("Closing subscription");
	// 		futures::future::ok(Value::Bool(true))
	// 	}),
	// );

	// let handler: MetaIoHandler<_> = io.into();

	// let server = ServerBuilder::with_meta_extractor(handler, |context: &RequestContext| {
	// 	Arc::new(Session::new(context.out.clone()))
	// })
	let server = ServerBuilder::with_meta_extractor(io, |context: &RequestContext| {
		Arc::new(Session::new(context.sender().clone()))
	})
	.start(&"0.0.0.0:4445".parse().unwrap())
	.expect("Unable to start RPC server");

	server.wait();
}
