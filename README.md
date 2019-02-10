# Okeydokey Script Profile Manager

This is a tool for building directory script profiles in `.ok` files. The idea
is to store commonly used scripts in a lightweight format for personal use.
Inspired by [SecretGeek](http://www.secretgeek.net/ok).

## What
Okeydokey is a script profile manager which will walk up the directory tree
searching for a .ok file. When found, it will either return the command
associated with the passed in prefix, or return the command names in a space
separated list.

This tool is intended to be used in combination with a helper function in the
shell script of the user's choice. Mine is built in PowerShell:

```Powershell
function ok
{
  Param($command = $null)
  if ($command -eq $null) {
    okeydokey
  } else {
    $script = okeydokey $command -p "pushd {};" -s "; popd"
    if ($script -ne $null) {
      iex $script
    }
  }
}
```

The `-p` argument stands for prefix, and the `-s` argument stands for sufix. They will added to the output command and the `{}` holes will be filled with the path to the directory containing the `.ok` file. If no `.ok` file is found, no output will be written.

## Why

Frequently I find build systems and other script managers too heavy weight to
use for all of the little things I need to keep track of. By ignoring the cross
platform support of utility functions, and assuming that the functions will only
be used by the author we are able to build a profile system with very simple
rules that is easy to use and understand

This tool is heavily based upon the ideas put forth in SecretGeek's
[article](http://www.secretgeek.net/ok) however Okeydokey makes a couple of
improvements. By writing it in Rust and doing the heavy lifting there we get
cross platform support for free. Okeydokey walks up the directory tree searching
for the `.ok` file freeing the user up from making sure they are in the correct
place. Okeydokey also names arguments and outputs them instead of the numbered
file. I argue this helps the user remember which command to run without clogging
their console up with unnecessary details.

## Dev Log

Okeydokey is a part of my push to make tangible and documented progress on a project every day.

[Day2](http://02credits.com/blog/day2-okeydokey)
[Day3](http://02credits.com/blog/day3-okeydokey-cont)
