extern crate mackerel_plugin;
extern crate mackerel_plugin_ntp;

use mackerel_plugin::Plugin;
use mackerel_plugin_ntp::NtpPlugin;

use std::io::Cursor;

#[test]
fn plugin_output_values() {
    let plugin = NtpPlugin {};
    let mut out = Cursor::new(Vec::new());
    assert_eq!(plugin.output_values(&mut out).is_ok(), true);
    let out_str = String::from_utf8(out.into_inner()).unwrap();
    assert_eq!(out_str.contains("ntp.poll.poll\t"), true);
    assert_eq!(out_str.contains("ntp.poll.when\t"), true);
    assert_eq!(out_str.contains("ntp.offset.offset\t"), true);
    assert_eq!(out_str.contains("ntp.delay.delay\t"), true);
    assert_eq!(out_str.contains("ntp.jitter.jitter\t"), true);
}

#[test]
fn plugin_output_definitions() {
    let plugin = NtpPlugin {};
    let mut out = Cursor::new(Vec::new());
    assert_eq!(plugin.output_definitions(&mut out).is_ok(), true);
    let out_str = String::from_utf8(out.into_inner()).unwrap();
    assert_eq!(out_str.starts_with("# mackerel-agent-plugin\n"), true);
}
