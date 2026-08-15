use dk::commands;

fn main() {
    let config = dkb_core::config::Config::load().unwrap_or_default();
    let data_dir = config.data_dir.clone();

    if let Err(e) = dkb_core::storage::Storage::init(&data_dir) {
        eprintln!("dk: failed to initialize data dir: {e}");
        std::process::exit(1);
    }

    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("usage: dk <command> [args...]");
        eprintln!("commands: new, list, pick, edit, move, delete");
        std::process::exit(1);
    }

    let cmd = args[0].as_str();
    let rest = &args[1..];

    let result = match cmd {
        "new" | "n" => {
            commands::new::run_new(&data_dir, rest.first().map(String::as_str)).map(|_| ())
        }
        "list" | "ls" => {
            commands::list::run_list(&data_dir, rest.first().map(String::as_str))
        }
        "pick" | "p" => {
            if rest.is_empty() {
                Err(std::io::Error::other("pick requires an argument"))
            } else {
                commands::pick::run_pick(&data_dir, &rest[0]).map(|_| ())
            }
        }
        "edit" | "ed" => commands::edit::run_edit(&data_dir, rest),
        "move" | "mv" => {
            if rest.len() < 2 {
                Err(std::io::Error::other("move requires <selection> <category>"))
            } else {
                commands::move_cmd::run_move(&data_dir, &rest[0], &rest[1])
            }
        }
        "delete" | "rm" => {
            let mut force = false;
            let mut filtered: Vec<String> = Vec::new();
            for a in rest {
                if a == "-f" || a == "--force" {
                    force = true;
                } else {
                    filtered.push(a.clone());
                }
            }
            commands::delete::run_delete(&data_dir, &filtered, force)
        }
        other => Err(std::io::Error::other(format!("unknown command: {other}"))),
    };

    if let Err(e) = result {
        eprintln!("dk: {e}");
        std::process::exit(1);
    }
}
