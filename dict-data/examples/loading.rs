extern crate dict_data;

fn main() {
	println!("=> Loading database ~ v{}", dict_data::version());

	let start = std::time::Instant::now();
	dict_data::load();
	println!("-> Loaded in {:?}", start.elapsed());
	println!();
}
