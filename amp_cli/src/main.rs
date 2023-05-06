mod apps;
use apps::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    AmpTool::new().run()
}
