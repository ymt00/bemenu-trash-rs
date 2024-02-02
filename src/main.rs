use chrono::NaiveDateTime;
use std::{
    collections::BTreeMap,
    env, fs,
    io::BufRead,
    path::Path,
    process::{Command, Output, Stdio},
};
use sway::bemenu;

const TRASH_LIST_BIN: &str = "/usr/bin/trash-list";
const TRASH_RESTORE_BIN: &str = "/usr/bin/trash-restore";
const ECHO_BIN: &str = "/usr/bin/echo";
const BEMENU_ARGS: [&str; 5] = ["--list", "40", "--no-exec", "-p", " Poubelle"];
const YAD_BIN: &str = "/usr/bin/yad";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("You must provide a path to your trash directory.");
    }

    let trash_dir: &str = &args[1];

    let datetime_path_map: BTreeMap<i64, String> = get_datetime_path_map();

    let items: String = datetime_path_map
        .iter()
        .map(|f: (&i64, &String)| {
            let dt: NaiveDateTime = NaiveDateTime::from_timestamp_opt(f.0.to_owned(), 0).unwrap();

            let path: &Path = Path::new(f.1);

            let metadata: Result<fs::Metadata, std::io::Error> = fs::metadata(
                trash_dir.to_owned() + "/files/" + path.file_name().unwrap().to_str().unwrap(),
            );

            let icon: &str = if metadata.expect("Failed to get metadata").is_dir() {
                "\u{f07b}"
            } else {
                "\u{f15b}"
            };

            format!(
                "{}\t{} {}\n",
                dt.format("\u{f133} %a %d %b %Y \u{f017} %H:%M"),
                icon,
                f.1
            )
        })
        .collect::<String>();

    if items.trim() == "" {
        bemenu("La poubelle est vide", &BEMENU_ARGS);
    } else {
        let sel: String = bemenu(items.trim_end_matches('\n'), &BEMENU_ARGS);

        if !sel.is_empty() {
            let rest_file: &str = sel.split_once('\t').unwrap().1.split_once(' ').unwrap().1;

            let output: Output = Command::new(YAD_BIN)
                .args([
                    "--name=\"floating 420x180\"",
                    format!(
                        "--text=Voulez-vous vraiment restaurer <b>{}</b> ?",
                        rest_file
                    )
                    .as_str(),
                ])
                .output()
                .expect("Command failed");

            if output.status.success() {
                Command::new(TRASH_RESTORE_BIN)
                    .arg(rest_file)
                    .stdin(Stdio::from(
                        Command::new(ECHO_BIN)
                            .arg("0")
                            .stdout(Stdio::piped())
                            .spawn()
                            .unwrap()
                            .stdout
                            .unwrap(),
                    ))
                    .stdout(Stdio::null())
                    .spawn()
                    .expect("Command failed");
            }
        }
    }
}

fn get_datetime_path_map() -> BTreeMap<i64, String> {
    let output = Command::new(TRASH_LIST_BIN)
        .output()
        .expect("Command failed");

    output
        .stdout
        .lines()
        .map(|l| {
            let line = l.unwrap();
            let (date_time, path) = line.split_at(19);
            (
                NaiveDateTime::parse_from_str(date_time, "%Y-%m-%d %H:%M:%S")
                    .unwrap()
                    .timestamp(),
                path.trim_start().to_string(),
            )
        })
        .collect()
}
