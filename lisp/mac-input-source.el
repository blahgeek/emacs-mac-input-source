;;; mac-input-source.el --- Provide helper functions related to macOS input sources  -*- lexical-binding: t; -*-

;; Copyright (C) 2025  Yikai Zhao

;; Author: Yikai Zhao <yikai@z1k.dev>
;; Keywords: macos, i18n

;; This program is free software; you can redistribute it and/or modify
;; it under the terms of the GNU General Public License as published by
;; the Free Software Foundation, either version 3 of the License, or
;; (at your option) any later version.

;; This program is distributed in the hope that it will be useful,
;; but WITHOUT ANY WARRANTY; without even the implied warranty of
;; MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
;; GNU General Public License for more details.

;; You should have received a copy of the GNU General Public License
;; along with this program.  If not, see <https://www.gnu.org/licenses/>.

;;; Commentary:

;;

;;; Code:

;; (add-to-list 'load-path ".../emacs-mac-input-source/target/debug/")
(require 'mac-input-source-dyn)

(defun mac-input-source--new (source)
  (pcase source
    ((or 'nil 'keyboard)
     (mac-input-source--new-current-keyboard))
    ('keyboard-layout
     (mac-input-source--new-current-keyboard-layout))
    ('ascii-capable-keyboard
     (mac-input-source--new-current-ascii-capable-keyboard))
    ('ascii-capable-keyboard-layout
     (mac-input-source--new-current-ascii-capable-keyboard-layout))
    ('keyboard-layout-override
     (mac-input-source--new-input-method-keyboard-layout-override))
    ('t
     (error "Getting input source by locale is not supported yet"))
    ((pred stringp)
     (car (or (mac-input-source--new-list source nil)
              (mac-input-source--new-list source 'include-all-installed))))))

(defun mac-input-source--format (input-source format)
  (let* ((props (mac-input-source--get-properties input-source))
         (id (plist-get props :id)))
    (pcase format
      ('nil id)
      ('t (cons id props))
      ((pred keywordp) (cons id (plist-get props format)))
      (_ (error "Format %s not supported" format)))))

(defun mac-input-source (&optional source format)
  "Return ID optionally with properties of input source SOURCE.
Optional 1st arg SOURCE specifies an input source.  It can be a symbol
or a string.  If it is a symbol, it has the following meaning:

nil or `keyboard'
    The currently-selected keyboard input source.
`keyboard-layout'
    The keyboard layout currently being used.
`ascii-capable-keyboard'
    The most-recently-used ASCII-capable keyboard input source.
`ascii-capable-keyboard-layout'
    The most-recently-used ASCII-capable keyboard layout.
`keyboard-layout-override'
    Currently-selected input method's keyboard layout override.
    This may return nil.

If SOURCE is a string, it is interpreted as an input source ID,
which should be an element of the result of `(mac-input-source-list
t)'.  Return nil if the specified input source ID does not exist.

Optional 2nd arg FORMAT must be a symbol and controls the format of the result.

If FORMAT is nil or unspecified, then the result is a string of input
source ID, which is the unique reverse DNS name associated with the
input source.

If FORMAT is t, then the result is a cons (ID . PLIST) of an input
source ID string and a property list containing the following names
and values:

`:category'
    The category of input source.  The possible values are
    \"TISCategoryKeyboardInputSource\", \"TISCategoryPaletteInputSource\",
    and \"TISCategoryInkInputSource\".
`:type'
    The specific type of input source.  The possible values are
    \"TISTypeKeyboardLayout\", \"TISTypeKeyboardInputMethodWithoutModes\",
    \"TISTypeKeyboardInputMethodModeEnabled\",
    \"TISTypeKeyboardInputMode\", \"TISTypeCharacterPalette\",
    \"TISTypeKeyboardViewer\", and \"TISTypeInk\".
`:ascii-capable-p'
    Whether the input source identifies itself as ASCII-capable.
`:enable-capable-p'
    Whether the input source can ever be programmatically enabled.
`:select-capable-p'
    Whether the input source can ever be programmatically selected.
`:enabled-p'
    Whether the input source is currently enabled.
`:selected-p'
    Whether the input source is currently selected.
`:bundle-id'
    The reverse DNS BundleID associated with the input source.
`:input-mode-id'
    A particular usage class for input modes.
`:localized-name'
    The localized name for UI purposes.
`:languages'
    Codes for languages that can be input using the input source.
    Languages codes are in the BCP 47 format.  The first element is
    the language for which the input source is intended.

The value corresponding to a name ending with \"-p\" is nil or t.  The
value for `:languages' is a vector of strings.  The other values are
strings.

If FORMAT is a symbol, then it is interpreted as a property above and
the result is a cons (ID . VALUE) of an input source ID string and a
value corresponding to the property.

[XXX] Difference from mituharu/emacs-mac:

- SOURCE being a string as a language id is not supported.
- SOURCE being t to find the input source of current language is not supported.
- FORMAT being a list of symbols is not supported.
- Property :icon-image-file is not supported."
  (let ((input-source (mac-input-source--new source)))
    (mac-input-source--format input-source format)))

(defun mac-input-source-list (&optional type format)
  "Return a list of input sources.
If optional 1st arg TYPE is nil or unspecified, then all enabled input
sources are listed.  If TYPE is `ascii-capable-keyboard', then all
ASCII compatible enabled input sources are listed.  If TYPE is t, then
all installed input sources, whether enabled or not, are listed, but
this can have significant memory impact.

Optional 2nd arg FORMAT controls the format of the result.
See `mac-input-source' for their meanings."
  (let ((input-sources (pcase type
                         ('nil
                          (mac-input-source--new-list nil nil))
                         ('ascii-capable-keyboard
                          (mac-input-source--new-ascii-capable-list))
                         ('t
                          (mac-input-source--new-list nil 'include-all-installed))
                         (_ (error "Type %s not supported" type)))))
    (mapcar (lambda (x) (mac-input-source--format x format)) input-sources)))

(defun mac-select-input-source (source &optional set-keyboard-layout-override-p)
  "Select the input source SOURCE.
SOURCE is either a symbol or a string (see `mac-input-source').
Specifying nil results in re-selecting the current keyboard input
source and thus that is not meaningful.  So, unlike
`mac-input-source', SOURCE is not optional.

If optional 2nd arg SET-KEYBOARD-LAYOUT-OVERRIDE-P is non-nil, then
SOURCE is set as the keyboard layout override rather than the new
current keyboard input source.

Return t if SOURCE could be successfully selected.  Otherwise, return
nil."
  (when-let* ((input-source (mac-input-source--new source)))
    (condition-case nil
        (prog1 t
          (if set-keyboard-layout-override-p
              (mac-input-source--set-keyboard-layout-override input-source)
            (mac-input-source--select input-source)))
      (error nil))))

(defun mac-deselect-input-source (source)
  "Deselect the input source SOURCE.
This function is only intended for use with palette or ink input
sources; calling it has no effect on other input sources.  So, unlike
`mac-select-input-source', specifying a symbolic SOURCE other than t
causes an error.  SOURCE must be t or a string, and cannot be omitted.

Return t if SOURCE could be successfully deselected.  Otherwise,
return nil."
  (when-let* ((input-source (mac-input-source--new source)))
    (condition-case nil
        (prog1 t
          (mac-input-source--deselect input-source))
      (error nil))))

(provide 'mac-input-source)
;;; mac-input-source.el ends here
