# Emacs `mac-input-source` by dynamic module

Provide input source related functions for macOS via dynamic module.
These functions can be used for querying and switching input sources in emacs.

## Background

There's several ways to query or switch input sources in macOS in Emacs:

1. Command line tools like [fcitx-remote-for-osx](https://github.com/xcodebuild/fcitx-remote-for-osx) using `TISSelectInputSource` API.
However, they does not work properly when switching to CJKV input sources due to a macOS bug.
2. Command line tool [macism](https://github.com/laishulu/macism).
It overcomes the above bug by emulating keyboard shortcuts, which seems unreliable and hacky to me.
3. [Mituharu emacs-mac port](https://bitbucket.org/mituharu/emacs-mac/) supports `mac-input-source`, `mac-select-input-source` functions
natively. These functions also uses `TISSelectInputSource` API but they work reliably, because
apparently the bug does not happen when the API is calling from the application itself [ref](https://github.com/pqrs-org/Karabiner-Elements/issues/1602#issuecomment-605628367).

The emacs-mac port seems to be the best option here.
However, that project does not seem to be very active, neither does it provide up-to-date emacs versions.

This package provides compatible functions (`mac-input-source`, `mac-select-input-source`, etc.)
as those in the emacs-mac port by dynamic module, so they can be used with upstream version of emacs.

## Install

> TODO: This package has not been submitted to MELPA yet, manual install required.
> Pre-built binary not available yet, rust toolchain required for building.

- Clone the repo and add `".../emacs-mac-input-source/lisp/"` to `load-path`
- Build the dynamic module written in rust: `cargo build --release`
- Copy and rename the built library: `cd lisp; ln -s ../target/release/libmac_input_source_dyn.dyl
ib mac-input-source-dyn.dylib`

## Functions and difference between emacs-mac port

Provided functions:

- `mac-input-source`
- `mac-input-source-list`
- `mac-select-input-source`
- `mac-deselect-input-source`

See their function documentation for usage details.

The function names and funtionalities are almost exact same as the emacs-mac port,
with a few minor unsupported features, which are described in function documentation.
Full compatibility is expected for most use cases.

## Usage example with other packages

### [emacs-smart-input-source](https://github.com/laishulu/emacs-smart-input-source)

Just use `'emp` method. Everything should just work.

### [fcitx.el](https://github.com/cute-jumper/fcitx.el)

(I personally prefer fcitx.el over emacs-smart-input-source because it have better support for evil mode)

```
(define-advice fcitx--activate (:override () macos)
    (mac-select-input-source "com.apple.inputmethod.SCIM.Shuangpin"))
(define-advice fcitx--deactivate (:override () macos)
    (mac-select-input-source 'ascii-capable-keyboard))
(define-advice fcitx--active-p (:override () macos)
    (null (cdr (mac-input-source nil :ascii-capable-p))))
```
