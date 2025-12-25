mod controllers; // this ensures all controller files are compiled and registered
use rspring::Application;

fn main() {
    Application::new()
        .run();
}