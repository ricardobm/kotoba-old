/// Maintains the global application state for the application.
pub struct App {}

impl App {
	/// Initializes the application state and returns the static [App] instance.
	pub fn get() -> &'static App {
		lazy_static! {
			static ref APP: App = {
				let app = App {};
				app
			};
		}
		&APP
	}
}
