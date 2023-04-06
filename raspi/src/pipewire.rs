use color_eyre::Result;
use pipewire as pw;
use pw::{prelude::*, properties, spa};

fn init() -> Result<()> {
	pw::init();

	let main_loop = pw::MainLoop::new()?;
	let context = pw::Context::new(&main_loop)?;
	let core = context.connect(None)?;

	let main_loop_weak = main_loop.downgrade();
	core.add_listener_local()
		.error(move |id, seq, res, message| {
			eprintln!("error id:{} seq:{} res:{}: {}", id, seq, res, message);

			if id == 0 {
				if let Some(main_loop) = main_loop_weak.upgrade() {
					main_loop.quit();
				}
			}
		})
		.register();

	// let info = spa::pod::

	let stream = pw::stream::Stream::<i32>::with_user_data(
		&main_loop,
		"capture-test",
		properties! {
			*pw::keys::MEDIA_TYPE => "Audio",
			*pw::keys::MEDIA_CATEGORY => "Monitor",
			*pw::keys::MEDIA_ROLE => "Music",
		},
		0,
	)
	.state_changed(|old, new| {
		println!("State changed: {:?} -> {:?}", old, new);
	})
	.process(|stream, frame_count| {
		println!("On frame");
		match stream.dequeue_buffer() {
			None => println!("No buffer received"),
			Some(mut buffer) => {
				let datas = buffer.datas_mut();
				println!("Frame {}. Got {} datas.", frame_count, datas.len());
				*frame_count += 1;
				// TODO: get the frame size and display it
			}
		}
	})
	// TODO: connect params_changed
	.create()?;

	println!("Created stream {:#?}", stream);

	// TODO: set params

	stream.connect(
		spa::Direction::Input,
		opt.target,
		pw::stream::StreamFlags::AUTOCONNECT | pw::stream::StreamFlags::MAP_BUFFERS,
		&mut [],
	)?;

	println!("Connected stream");

	main_loop.run();

	unsafe { pw::deinit() };

	// main_loop.add_signal_local()

	Ok(())
}
