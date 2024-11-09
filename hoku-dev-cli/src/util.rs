use colored::ColoredString;
use regex::Regex;
use std::thread;
use std::thread::JoinHandle;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};

use crate::LogLevel;

pub fn filtered_print(
    label: &ColoredString,
    words: String,
    filters: &Vec<Result<Regex,regex::Error>>,
    log_level: LogLevel
) -> () {
    let mut mtch = false;
    for i in 0..filters.len() {
        if matches!(log_level, LogLevel::Debug) {
            break;
        }
        let filter = filters[i].as_ref().unwrap();
        if mtch {
            break;
        }
        if filter.is_match(&words) {
            mtch = true;
        }
    }
    if !mtch {
        println!("[{}] {}", label, words);
    }
}

pub fn pipe_sub_command(
    title: ColoredString,
    cmd: &str,
    args: Vec<&str>,
    envs: Option<Vec<Vec<&str>>>,
    current_dir: Option<&str>,
    out_filters: Vec<Result<Regex, regex::Error>>,
    err_filters: Vec<Result<Regex, regex::Error>>,
    log_level: &LogLevel
) -> (JoinHandle<()>, JoinHandle<()>) {
    let lblout = title.clone();
    let lblerr = title.clone();

    let mut cmd: &mut Command = &mut Command::new(cmd);

    if let Some(envs) = envs {
        for i in 0..(envs.len()) {
            if envs[i].len() == 2 {
                cmd = cmd.env(envs[i][0], envs[i][1]);
            }
        }
    }
    if let Some(current_dir) = current_dir {
        cmd.current_dir(current_dir);
    }

    let mut command_out = cmd
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stdout = command_out.stdout.take().expect(&format!("could not take {} stdout", lblout));
    let stderr = command_out.stderr.take().expect(&format!("could not take {} stderr", lblerr));

    // let (stdout_tx, stdout_rx) = std::sync::mpsc::channel();
    // let (stderr_tx, stderr_rx) = std::sync::mpsc::channel();

    let out_log = log_level.clone();
    let stdout_thread = thread::spawn(move || {
        let stdout_lines = BufReader::new(stdout).lines();
        for line in stdout_lines {
            let line = line.unwrap();
            filtered_print(&lblout, line, &out_filters, out_log);
        }
    });

    // TODO: all of our make tasks pipe info through stderr, figure out why
    let err_log = log_level.clone();
    let stderr_thread = thread::spawn(move || {
        let stderr_lines = BufReader::new(stderr).lines();
        for line in stderr_lines {
            let line = line.unwrap();
            filtered_print(&lblerr, line, &err_filters, err_log);
        }
    });

    // let status = command_out
    //     .wait()
    //     .expect("Internal error, failed to wait on child");

    return (stdout_thread, stderr_thread);
    // let stdout = stdout_rx.into_iter().collect::<Vec<String>>().join("");
    // let stderr = stderr_rx.into_iter().collect::<Vec<String>>().join("");

}
