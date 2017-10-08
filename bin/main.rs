extern crate mackerel_plugin;
extern crate mackerel_plugin_ntp;

use mackerel_plugin::Plugin;
use mackerel_plugin_ntp::NtpPlugin;

fn main() {
    let plugin = NtpPlugin {};
    match plugin.run() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("mackerel-plugin-ntp: {}", err);
            std::process::exit(1);
        }
    }
}
