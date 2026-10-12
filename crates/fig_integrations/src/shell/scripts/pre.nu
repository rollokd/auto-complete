mkdir ~/.local/bin | ignore

def pathadd [path: string] {
  if not ($env.PATH | any {|it| $it == $path }) {
    $env.PATH | prepend $path
  } else {
    $env.PATH
  }
}

let-env PATH = pathadd $"($env.HOME)/.local/bin"
let-env PATH = pathadd $"($env.HOME)/.local/bin"

if "AUTO_NEW_SESSION" in $env {
  let-env AUTOTERM_SESSION_ID = $nothing
  let-env AUTO_TERM = $nothing
  let-env AUTO_NEW_SESSION = $nothing
}

if "AUTO_SET_PARENT_CHECK" not-in $env {
  if "AUTO_PARENT" not-in $env and "AUTO_SET_PARENT" in $env {
    let-env AUTO_PARENT = $env.AUTO_SET_PARENT
    let-env AUTO_SET_PARENT = $nothing
  }
  let-env AUTO_SET_PARENT_CHECK = 1
}


let result = (^{{CLI_BINARY_NAME}} _ should-figterm-launch | complete)
let-env SHOULD_AUTOTERM_LAUNCH = $result.exit_code

let should_launch = (
    ("PROCESS_LAUNCHED_BY_AUTO" not-in $env or ($env.PROCESS_LAUNCHED_BY_AUTO | str length) == 0)
    and ($env.SHOULD_AUTOTERM_LAUNCH == 0 or
       ($env.SHOULD_AUTOTERM_LAUNCH == 2 and "AUTO_TERM" not-in $env))
)

if $should_launch {
  let AUTO_SHELL = ({{CLI_BINARY_NAME}} _ get-shell | complete).stdout
  
  let fig_term_name = "nu (figterm)"
  let figterm_path = if ([$env.HOME ".fig" "bin" $fig_term_name] | path join | path exists) {
    [$env.HOME ".fig" "bin" $fig_term_name] | path join
  } else if (which figterm | length) > 0 {
    which figterm | first | get path
  } else {
    [$env.HOME ".fig" "bin" "figterm"] | path join
  }

  with-env {
    AUTO_SHELL: $AUTO_SHELL
  } {
    exec $figterm_path
  }
}
