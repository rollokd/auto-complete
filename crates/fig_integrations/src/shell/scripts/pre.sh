#!/usr/bin/env bash

mkdir -p "${HOME}/.local/bin" > /dev/null 2>&1

# add ~/.local/bin to PATH
if [[ -d "${HOME}/.local/bin" ]] && [[ ":$PATH:" != *":${HOME}/.local/bin:"* ]]; then
  PATH="${PATH:+"$PATH:"}${HOME}/.local/bin"
fi

if [[ -n "${AUTO_NEW_SESSION:-}" ]]; then
  unset AUTOTERM_SESSION_ID
  unset AUTO_TERM
  unset AUTO_NEW_SESSION
fi

if [[ -z "${AUTO_SET_PARENT_CHECK:-}" ]]; then
  # Load parent from env variables
  if [[ -z "${AUTO_PARENT:-}" && -n "${AUTO_SET_PARENT:-}" ]]; then
    export AUTO_PARENT="$AUTO_SET_PARENT"
    unset -v AUTO_SET_PARENT
  fi
  export AUTO_SET_PARENT_CHECK=1
fi

# 0 = Yes, 1 = No, 2 = Fallback to AUTO_TERM
if [ -z "${SHOULD_AUTOTERM_LAUNCH:-}" ]; then
  {{CLI_BINARY_NAME}} _ should-figterm-launch 1>/dev/null 2>&1
  SHOULD_AUTOTERM_LAUNCH=$?
fi

# Only launch figterm if current session is not already inside PTY and command exists.
# PWSH var is set when launched by `pwsh -Login`, in which case we don't want to init.
# It is not necessary in Fish.
if   [[ -t 1 ]] \
  && [[ -z "${PROCESS_LAUNCHED_BY_AUTO:-}" ]] \
  && command -v {{PTY_BINARY_NAME}} 1>/dev/null 2>&1 \
  && [[ ("${SHOULD_AUTOTERM_LAUNCH}" -eq 0) || (("${SHOULD_AUTOTERM_LAUNCH}" -eq 2) && (-z "${AUTO_TERM:-}" || (-z "${AUTO_TERM_TMUX:-}" && -n "${TMUX:-}"))) ]]
then
  # Pty module sets AUTO_TERM or AUTO_TERM_TMUX to avoid running twice.
  if [ -z "${AUTO_SHELL:-}" ]; then
    AUTO_SHELL=$({{CLI_BINARY_NAME}} _ get-shell)
  fi
  AUTO_IS_LOGIN_SHELL="${AUTO_IS_LOGIN_SHELL:='0'}"

  # shellcheck disable=SC2030
  if ([[ -n "${BASH:-}" ]] && shopt -q login_shell) \
    || [[ -n "${ZSH_NAME:-}" && -o login ]]; then
    AUTO_IS_LOGIN_SHELL=1
  fi

  # Do not launch figterm in non-interactive shells (like VSCode Tasks)
  if [[ $- == *i* ]]; then
    AUTO_TERM_NAME="$(basename "${AUTO_SHELL}") ({{PTY_BINARY_NAME}})"
    if [[ -z "${AUTO_TERM_PATH:-}" ]]; then
      if [[ -x "${HOME}/.local/bin/${AUTO_TERM_NAME}" ]]; then
        AUTO_TERM_PATH="${HOME}/.local/bin/${AUTO_TERM_NAME}"
      else
        AUTO_TERM_PATH="$(command -v {{PTY_BINARY_NAME}} || echo "${HOME}/.local/bin/{{PTY_BINARY_NAME}}")"
      fi
    fi

    AUTO_EXECUTION_STRING="${BASH_EXECUTION_STRING:=$ZSH_EXECUTION_STRING}"

    # Get initial text.
    INITIAL_TEXT=""
    # shellcheck disable=SC2031
    if [[ -z "${BASH:-}" || "${BASH_VERSINFO[0]}" -gt "3" ]]; then
      while read -rt 0; do
        if [[ -n "${BASH:-}" ]]; then
          read -r
        fi
        INITIAL_TEXT="${INITIAL_TEXT}${REPLY}\n"
      done
    fi
    AUTO_EXECUTION_STRING="${AUTO_EXECUTION_STRING}" AUTO_START_TEXT="$(printf "%b" "${INITIAL_TEXT}")" AUTO_SHELL="${AUTO_SHELL}" AUTO_IS_LOGIN_SHELL="${AUTO_IS_LOGIN_SHELL}" exec -a "${AUTO_TERM_NAME}" "${AUTO_TERM_PATH}"
  fi
fi
