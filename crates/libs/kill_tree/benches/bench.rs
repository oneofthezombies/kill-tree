#![feature(test)]

extern crate test;

use kill_tree::{get_available_max_process_id, Config};
use test::Bencher;

#[bench]
fn kill_tree(b: &mut Bencher) {
    b.iter(|| {
        let target_process_id = get_available_max_process_id();
        kill_tree::blocking::kill_tree(target_process_id).unwrap();
    });
}

#[bench]
fn kill_tree_with_sigkill(b: &mut Bencher) {
    b.iter(|| {
        let target_process_id = get_available_max_process_id();
        let config = Config {
            signal: String::from("SIGKILL"),
            ..Default::default()
        };
        kill_tree::blocking::kill_tree_with_config(target_process_id, &config).unwrap();
    });
}

#[bench]
fn kill_tree_exclude_target(b: &mut Bencher) {
    b.iter(|| {
        let target_process_id = get_available_max_process_id();
        let config = Config {
            include_target: false,
            ..Default::default()
        };
        kill_tree::blocking::kill_tree_with_config(target_process_id, &config).unwrap();
    });
}
