use dk::commands;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        print_usage();
        std::process::exit(1);
    }

    let cmd = args[0].as_str();
    let rest = &args[1..];

    match cmd {
        "help" | "--help" | "-h" => {
            if rest.is_empty() {
                print_usage();
            } else {
                print_command_help(&rest[0]);
            }
            std::process::exit(0);
        }
        "new" | "n" | "list" | "ls" | "pick" | "p" | "edit" | "ed" | "move" | "mv"
        | "delete" | "rm" | "show" | "s" => {}
        other => {
            eprintln!("dk: unknown command: {other}");
            eprintln!();
            print_usage();
            std::process::exit(1);
        }
    }

    let config = dkb_core::config::Config::load().unwrap_or_default();
    let data_dir = config.data_dir.clone();

    if let Err(e) = dkb_core::storage::Storage::init(&data_dir) {
        eprintln!("dk: failed to initialize data dir: {e}");
        std::process::exit(1);
    }

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
        "show" | "s" => commands::show::run_show(&data_dir, rest),
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
        _ => unreachable!(),
    };

    if let Err(e) = result {
        eprintln!("dk: {e}");
        std::process::exit(1);
    }
}

fn print_usage() {
    eprintln!("dk — Daily Kanban command-line tool");
    eprintln!();
    eprintln!("usage: dk <command> [args...]");
    eprintln!();
    eprintln!("commands:");
    eprintln!("  new (n)         Create a new item via $EDITOR");
    eprintln!("  list (ls)       List items with indices");
    eprintln!("  pick (p)        Set the current item");
    eprintln!("  edit (ed)       Edit one or more items via $EDITOR");
    eprintln!("  show (s)        Display an item via $PAGER or stdout");
    eprintln!("  move (mv)       Move an item to a different category");
    eprintln!("  delete (rm)    Delete items (with confirmation)");
    eprintln!("  help            Show this help, or help for a command");
    eprintln!();
    eprintln!("run 'dk help <command>' for details on a specific command");
}

fn print_command_help(cmd: &str) {
    match cmd {
        "new" | "n" => print_new_help(),
        "list" | "ls" => print_list_help(),
        "pick" | "p" => print_pick_help(),
        "edit" | "ed" => print_edit_help(),
        "move" | "mv" => print_move_help(),
        "delete" | "rm" => print_delete_help(),
        "show" | "s" => print_show_help(),
        "help" => print_help_help(),
        other => {
            eprintln!("dk: unknown command for help: {other}");
            eprintln!();
            print_usage();
        }
    }
}

fn print_new_help() {
    println!("dk new [category]    (alias: dk n)");
    println!();
    println!("Create a new item. Opens $VISUAL or $EDITOR with '# ' and the");
    println!("cursor positioned after it. By default the item is created in");
    println!("the backlog unless a category is given.");
    println!();
    println!("category aliases:");
    println!("  b, backlog         y, yesterday      t, today");
    println!("  tw, thisweek       nw, nextweek      d, done");
    println!();
    println!("examples:");
    println!("  dk n                   # new backlog item");
    println!("  dk new today           # new item in today");
    println!("  dk n tw                # new item in this week");
    println!("  dk new done            # new item already done");
}

fn print_list_help() {
    println!("dk list [category]    (alias: dk ls)");
    println!();
    println!("List items with their index numbers. The current item is");
    println!("marked with '*'. By default lists all active items");
    println!("(yesterday, today, this week, next week). Backlog and done");
    println!("are only shown when explicitly requested.");
    println!();
    println!("The index from the last 'dk list' is remembered so that");
    println!("'dk pick 3' and 'dk edit 3' refer to the right item.");
    println!();
    println!("examples:");
    println!("  dk ls                  # all active items");
    println!("  dk ls backlog          # backlog only");
    println!("  dk ls done             # done only");
    println!("  dk ls yesterday        # yesterday only");
}

fn print_pick_help() {
    println!("dk pick <selection>    (alias: dk p)");
    println!();
    println!("Set the current item marker. The selection can be:");
    println!("  <number>       index from the last 'dk list' output");
    println!("  <uuid>.md      filename of the item");
    println!("  <path>         full path to the item file");
    println!();
    println!("examples:");
    println!("  dk p 3                  # pick index 3");
    println!("  dk pick 550e8400....md  # pick by filename");
}

fn print_edit_help() {
    println!("dk edit [selection...]    (alias: dk ed)");
    println!();
    println!("Edit one or more items in $VISUAL or $EDITOR. With no");
    println!("arguments, edits the current item. Otherwise edits each");
    println!("selected item in sequence.");
    println!();
    println!("selection can be: a number, category/number, a bare");
    println!("category (acts on the current item), a uuid.md filename,");
    println!("or a full path.");
    println!();
    println!("examples:");
    println!("  dk ed                   # edit current item");
    println!("  dk edit 3               # edit index 3");
    println!("  dk edit 3 5 9           # edit three items");
    println!("  dk edit backlog/5        # edit 6th backlog item");
    println!("  dk edit today            # edit current item (bare category)");
}

fn print_move_help() {
    println!("dk move <selection> <category>    (alias: dk mv)");
    println!();
    println!("Move an item to a different category. Uses the same");
    println!("selection mechanism as edit.");
    println!();
    println!("examples:");
    println!("  dk mv 3 done            # move index 3 to done");
    println!("  dk mv yesterday/1 today  # move yesterday item 1 to today");
    println!("  dk mv backlog nextweek  # move first backlog item to next week");
}

fn print_delete_help() {
    println!("dk delete [selection...] [-f]    (alias: dk rm)");
    println!();
    println!("Delete one or more items. With no arguments, deletes the");
    println!("current item. Prompts for confirmation unless -f or");
    println!("--force is given. Uses the same selection mechanism as");
    println!("edit.");
    println!();
    println!("examples:");
    println!("  dk rm                   # delete current item (prompts)");
    println!("  dk rm 3                 # delete index 3 (prompts)");
    println!("  dk rm backlog/5         # delete 6th backlog item (prompts)");
    println!("  dk rm 3 5 9 -f          # delete three items without prompting");
}

fn print_help_help() {
    println!("dk help [command]");
    println!();
    println!("Show general help, or detailed help for a specific command.");
    println!();
    println!("available commands: new, list, pick, edit, show, move, delete");
}

fn print_show_help() {
    println!("dk show [selection]    (alias: dk s)");
    println!();
    println!("Display an item's content. Uses $PAGER (default: less) when");
    println!("output is a terminal. When piped, streams the raw content to");
    println!("stdout without invoking a pager.");
    println!();
    println!("With no argument, shows the current item. Selection uses the");
    println!("same mechanism as edit: a number, category/number, a bare");
    println!("category, a uuid.md filename, or a full path.");
    println!();
    println!("examples:");
    println!("  dk s                    # show current item");
    println!("  dk show 3               # show item at index 3");
    println!("  dk show backlog/0       # show first backlog item");
    println!("  dk show 550e8400....md  # show by filename");
}
