#[macro_use]
extern crate log;
use logging::{rule::RuleFilter, FastLoggerBuilder};

fn main() {
    FastLoggerBuilder::new()
        .set_level_from_str("Trace")
        // Let's log everything in white
        .add_rule_filter(RuleFilter::new(".*", false, "White"))
        .add_rule_filter(RuleFilter::new("Abort", false, "Red"))
        .add_rule_filter(RuleFilter::new("warned", false, "Yellow"))
        .add_rule_filter(RuleFilter::new("twice", true, "Yellow"))
        .build()
        .expect("Fail to instanciate the logging factory.");

    trace!("Track me if you can.");
    debug!("What's bugging you today?");
    info!(target: "Simple_example_instance_id", "Some interesting info here");
    warn!("You've been warned Sir!");
    // This next one will not be logged according to our defined rule
    warn!("Let's not warn twice about the same stuff.");
    // And this one will be printed in red
    error!("Abort the mission!!");

    // Let's give some time to the working thread to finish logging...
    std::thread::sleep(std::time::Duration::from_millis(10));
}
