command mkdir -p ~/.local/bin >/dev/null

builtin contains $HOME/.local/bin $PATH
or set --append PATH $HOME/.local/bin

if not test -z "$AUTO_NEW_SESSION"
    set --erase AUTOTERM_SESSION_ID
    set --erase AUTO_TERM
    set --erase AUTO_NEW_SESSION
end

# Load parent from env variables
if test -n "$AUTO_PARENT"; and test -z "$AUTO_SET_PARENT"
    set --export AUTO_PARENT $AUTO_SET_PARENT
end

if test -z "$SHOULD_AUTOTERM_LAUNCH"
    # 0 = Yes, 1 = No, 2 = Fallback to AUTO_TERM
    {{CLI_BINARY_NAME}} _ should-figterm-launch 1>/dev/null 2>&1
    set SHOULD_AUTOTERM_LAUNCH $status
end

if test -t 1
    and test -z "$PROCESS_LAUNCHED_BY_AUTO"
    and command -v {{PTY_BINARY_NAME}} 1>/dev/null 2>/dev/null
    and test "$SHOULD_AUTOTERM_LAUNCH" -eq 0 -o \( "$SHOULD_AUTOTERM_LAUNCH" -eq 2 -a \( -z "$AUTO_TERM" -o \( -z "$AUTO_TERM_TMUX" -a -n "$TMUX" \) \) \)

    if test -z "$AUTO_SHELL"
        set AUTO_SHELL ({{CLI_BINARY_NAME}} _ get-shell)
    end
    set AUTO_IS_LOGIN_SHELL 0
    if status --is-login
        set AUTO_IS_LOGIN_SHELL 1
    end

    # Do not launch {{PTY_BINARY_NAME}} in non-interactive shells (like VSCode Tasks)
    if status --is-interactive
        set AUTO_TERM_NAME (command basename "$AUTO_SHELL")" ({{PTY_BINARY_NAME}})"
        if not set -q AUTO_TERM_PATH
            if test -x "$HOME/.local/bin/$AUTO_TERM_NAME"
                set AUTO_TERM_PATH "$HOME/.local/bin/$AUTO_TERM_NAME"
            else
                set AUTO_TERM_PATH (command -v {{PTY_BINARY_NAME}} || echo "$HOME/.local/bin/{{PTY_BINARY_NAME}}")
            end
        end

        # Need to exec bash because we're using 'exec -a <name>'
        # to set argv[0] and fish's exec doesn't have this option
        exec bash -c "AUTO_PARENT=$AUTO_PARENT AUTO_SHELL=$AUTO_SHELL AUTO_IS_LOGIN_SHELL=$AUTO_IS_LOGIN_SHELL exec -a \"$AUTO_TERM_NAME\" \"$AUTO_TERM_PATH\""
    end
end
