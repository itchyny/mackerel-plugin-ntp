#[macro_use]
extern crate mackerel_plugin;

use mackerel_plugin::*;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};

struct NtpInfo {
    pub when: f64,
    pub poll: f64,
    pub reach: u32,
    pub delay: f64,
    pub offset: f64,
    pub jitter: f64,
}

fn get_ntp_info() -> Result<NtpInfo, String> {
    let child = Command::new("ntpq")
        .arg("-pn")
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to execute ntpq: {}", e))?;
    let stdout = child.stdout.ok_or("faild to read stdout of ntpq".to_string())?;
    let line = BufReader::new(stdout)
        .lines()
        .filter_map(|line_opt| line_opt.ok())
        .filter(|line| line.starts_with("*"))
        .next()
        .ok_or("failed to find ntp information in sync".to_string())?;
    let parts: Vec<_> = line.split_whitespace().collect();
    Ok(NtpInfo {
        when: parts.get(4).and_then(|x| x.parse().ok()).ok_or("failed to find when from ntpq -pn")?,
        poll: parts.get(5).and_then(|x| x.parse().ok()).ok_or("failed to find poll from ntpq -pn")?,
        reach: parts
            .get(6)
            .and_then(|x| x.parse().ok())
            .map(|x: u32| x.count_ones())
            .ok_or("failed to find reach from ntpq -pn")?,
        delay: parts.get(7).and_then(|x| x.parse().ok()).ok_or("failed to find delay from ntpq -pn")?,
        offset: parts.get(8).and_then(|x| x.parse().ok()).ok_or("failed to find offset from ntpq -pn")?,
        jitter: parts.get(9).and_then(|x| x.parse().ok()).ok_or("failed to find jitter from ntpq -pn")?,
    })
}

pub struct NtpPlugin {}

impl Plugin for NtpPlugin {
    fn fetch_metrics(&self) -> Result<HashMap<String, f64>, String> {
        let mut metrics = HashMap::new();
        let info = get_ntp_info()?;
        metrics.insert("poll.poll".to_string(), info.poll);
        metrics.insert("poll.when".to_string(), info.when);
        metrics.insert("reach.reach".to_string(), info.reach as f64);
        metrics.insert("delay.delay".to_string(), info.delay);
        metrics.insert("offset.offset".to_string(), info.offset.abs());
        metrics.insert("jitter.jitter".to_string(), info.jitter);
        Ok(metrics)
    }

    fn graph_definition(&self) -> Vec<Graph> {
        vec![
            graph! {
                name: "poll",
                label: "NTP poll",
                unit: "integer",
                metrics: [
                    { name: "poll", label: "poll (sec)" },
                    { name: "when", label: "when (sec)" },
                ]
            },
            graph! {
                name: "reach",
                label: "NTP reach",
                unit: "integer",
                metrics: [
                    { name: "reach", label: "reach" },
                ]
            },
            graph! {
                name: "delay",
                label: "NTP delay",
                unit: "float",
                metrics: [
                    { name: "delay", label: "delay (msec)" },
                ]
            },
            graph! {
                name: "offset",
                label: "NTP offset",
                unit: "float",
                metrics: [
                    { name: "offset", label: "offset (msec)" },
                ]
            },
            graph! {
                name: "jitter",
                label: "NTP jitter",
                unit: "float",
                metrics: [
                    { name: "jitter", label: "jitter (msec)" },
                ]
            },
        ]
    }

    fn metric_key_prefix(&self) -> String {
        "ntp".to_string()
    }
}
