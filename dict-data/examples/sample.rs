extern crate dict_data;

fn main() {
	println!();
	println!("Sample queries ~ v{}", dict_data::version());
	println!("==============");

	let start = std::time::Instant::now();
	dict_data::load();
	println!("-> Executed in {:?}", start.elapsed());

	println!("Bye!");
	println!();
}
