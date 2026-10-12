# Auto

Fig-style terminal autocomplete for macOS: a native popup next to your cursor for zsh in Ghostty and Zed, with a menubar icon for settings.

Forked from [aws/amazon-q-developer-cli-autocomplete](https://github.com/aws/amazon-q-developer-cli-autocomplete) (MIT OR Apache-2.0) with login, chat, telemetry and the updater removed. Completion specs are vendored from [withfig/autocomplete](https://github.com/withfig/autocomplete) (MIT) and themes from [withfig/themes](https://github.com/withfig/themes).

## Build

Requires rustup (honours `rust-toolchain.toml`), pnpm and node. Full Xcode is not needed.

```sh
pnpm app            # -> build/Auto.app
```

`SKIP_WEB=1` / `SKIP_SPECS=1` skip rebuilding the web apps / specs.

### Stable signing (optional)

macOS ties Accessibility and input method permissions to the code signature. Ad-hoc builds lose them on every rebuild. Create a local identity once and the build script picks it up automatically:

```sh
scripts/create-signing-cert.sh
```

## Install

1. Copy `build/Auto.app` to `/Applications` and open it.
2. Follow the onboarding: shell integration, Accessibility permission, input method.
3. Restart Ghostty / Zed so they load the input method.

## Layout

| Path | What |
|---|---|
| `crates/fig_desktop` | Menubar app, popup + settings windows (tao/wry) |
| `crates/figterm` | `autoterm`, the pty proxy that reads the zsh edit buffer |
| `crates/fig_input_method` | Input method used to find the caret position |
| `crates/q_cli` | `auto` CLI (shell init, integrations, settings) |
| `packages/autocomplete-app` | Popup UI (React) |
| `packages/dashboard-app` | Settings UI (React) |
| `specs/` | Completion specs, compiled into `Auto.app/Contents/Resources/specs` |
| `themes/` | Popup themes |

Custom specs: drop compiled `.js` specs into `~/Library/Application Support/auto/autocomplete/specs`; they override bundled ones.
