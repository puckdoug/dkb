# Command-line interface

Create a Command-line tool that can be used to interact with the same files as are managed in the GUI. The CLI tool should be named dk and process subcommands.

## Installation

There will need to be a button in configuration to install a symlink from the user's local bin (using OS default) to the cli.

## dk new / dk n

Create a new item. By default, it creates it in the backlog unless a category is added. Examples:

```
dk n
dk new
dk new yesterday
dk n y
dk n t
dk n tw
dk new thisweek
dk n nw
dk new nextweek
dk n d
dn new done
```

In each case, it will open a new file in the correct location using $VISUAL or $EDITOR with '# ' and the cursor at the end of the 2nd character.

## dk list / dk ls

With no arguments, lists all active items, but can take a category as argument (e.g. dk list yesterday) to filter. `dk list done` and `dk list backlog` are available to list them, but they are not displayed by default with just `dk ls`.

Each item in the list is preceeded by a number (0, 1, 2, 3...) right-justfied by column width followed by a space and as much of the header/subject (minus formatting) as is possible to present in the teminal window without wrapping. The row of the "current" item is preceeded with a '* ' to identify it. The current item can be updated with `dk pick #`

```
dk ls
dk ls backlog
dk ls done
dk ls yesterday
```

## dk pick / dk p

Sets the current marker to a particular item in the list, based on the number proided. This can accept the filename (uuid.md) or full path to a file, but typically will expect the number from dk list. This implies that the number from the _last_ run of dk list needs to be remembered so that dk pick will know which "3" applies.

## dk edit / dk ed

With no arguments, edits the current item. With an argument, edits item or items passed. examples:

```
dk ed
dk edit 3
dk edit 3 5 9
dk edit long-uuid-here.md
dk edit /some/path/to/filename-uuid.md
dk edit backlog/5
dk edit today
```

uses $VISUAL or $EDITOR.

## dk move / dk mv

Moves an item between boards or categories. Selection uses the same mechanism as edit.

```
dk mv 3 done
dk mv yesterday/1 today
dk mv backlog nextweek
```

## dk delete / dk rm

Deletes an item. Selection is the same as other systems.

```
dk rm
dk rm 3
dk rm done
dk rm backlog/5
```
