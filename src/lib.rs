#[macro_use]
extern crate mackerel_plugin;

use mackerel_plugin::*;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::str::FromStr;

#[derive(PartialEq, Debug)]
struct NtpInfo {
    pub when: Interval,
    pub poll: Interval,
    pub reach: Reach,
    pub delay: f64,
    pub offset: f64,
    pub jitter: f64,
}

#[derive(PartialEq, Debug)]
struct Interval {
    pub interval: u64,
}

impl FromStr for Interval {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Interval {
            interval: s.parse()
                .or(s.trim_right_matches('m').parse::<u64>().map(|m| m * 60))
                .or(s.trim_right_matches('h').parse::<u64>().map(|h| h * 60 * 60))
                .or(s.trim_right_matches('d').parse::<u64>().map(|d| d * 60 * 60 * 24))
                .map_err(|_| "failed to parse interval".to_string())?,
        })
    }
}

#[derive(PartialEq, Debug)]
struct Reach {
    pub reach: u32,
}

impl FromStr for Reach {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Reach {
            reach: s.chars().flat_map(|c| c.to_digit(8).map(|n| n.count_ones())).sum::<u32>(),
        })
    }
}

fn get_ntp_info() -> Result<NtpInfo, String> {
    let child = Command::new("ntpq")
        .arg("-pn")
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to execute ntpq: {}", e))?;
    let lines: Vec<_> = BufReader::new(child.stdout.ok_or("faild to read stdout of ntpq".to_string())?)
        .lines()
        .filter_map(|line_opt| line_opt.ok())
        .skip(2)
        .collect();
    let line = lines
        .iter()
        .filter(|line| line.starts_with("*"))
        .next()
        .or(lines.first())
        .ok_or("failed to find ntp information".to_string())?;
    parse_ntp_line(line.clone())
}

fn parse_ntp_line(line: String) -> Result<NtpInfo, String> {
    let parts: Vec<_> = line.split_whitespace().collect();
    Ok(NtpInfo {
        when: parts.get(4).and_then(|x| x.parse().ok()).ok_or("failed to find when from ntpq -pn")?,
        poll: parts.get(5).and_then(|x| x.parse().ok()).ok_or("failed to find poll from ntpq -pn")?,
        reach: parts.get(6).and_then(|x| x.parse().ok()).ok_or("failed to find reach from ntpq -pn")?,
        delay: parts.get(7).and_then(|x| x.parse().ok()).ok_or("failed to find delay from ntpq -pn")?,
        offset: parts.get(8).and_then(|x| x.parse().ok()).ok_or("failed to find offset from ntpq -pn")?,
        jitter: parts.get(9).and_then(|x| x.parse().ok()).ok_or("failed to find jitter from ntpq -pn")?,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_ntp_line() {
        let tests = vec![
            (
                " 17.253.68.253   .GPSs.           1 u  35m  512   16   51.191  831.748   8.311",
                NtpInfo {
                    when: Interval { interval: 2100 },
                    poll: Interval { interval: 512 },
                    reach: Reach { reach: 3 },
                    delay: 51.191,
                    offset: 831.748,
                    jitter: 8.311,
                },
            ),
            (
                " 2001:4860:4806: .GOOG.           1 u   29   64   377  86.892   -6.196   0.001",
                NtpInfo {
                    when: Interval { interval: 29 },
                    poll: Interval { interval: 64 },
                    reach: Reach { reach: 8 },
                    delay: 86.892,
                    offset: -6.196,
                    jitter: 0.001,
                },
            ),
        ];
        for (src, expected) in tests {
            assert_eq!(parse_ntp_line(src.to_string()), Ok(expected));
        }
    }
}

pub struct NtpPlugin {}

impl Plugin for NtpPlugin {
    fn fetch_metrics(&self) -> Result<HashMap<String, f64>, String> {
        let mut metrics = HashMap::new();
        let info = get_ntp_info()?;
        metrics.insert("poll.poll".to_string(), info.poll.interval as f64);
        metrics.insert("poll.when".to_string(), info.when.interval as f64);
        metrics.insert("reach.reach".to_string(), info.reach.reach as f64);
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
