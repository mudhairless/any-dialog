# Roadmap

Version 1.0 is Tier 1 below, complete for all four backends
(kdialog, zenity, osascript, and PowerShell on Windows). Tier 2 is
not part of 1.0. Version 0.1.0 was published early, on Oct 29, 2026,
to reserve the crate name; it carries only the three dialogs the
code started with.

Goal: cover as much of the dialog overlap between kdialog, zenity,
osascript (AppleScript Standard Additions), and PowerShell/WinForms
as the common API can honestly represent. Findings checked Oct 29,
2026 against the kdialog and zenity documentation and the
AppleScript command list; the PowerShell route was reevaluated on
Oct 30, 2026 (see the Windows section below).

## Current API (extracted from opencode-usage-tracker)

- Text entry: kdialog --inputbox, zenity --entry
- Single choice: kdialog --radiolist, zenity --list --radiolist
- Error message: kdialog --error, zenity --error

## Tier 1: all four backends map cleanly

Add these as common API, each kind landing in lockstep with a
rendering per backend (plus macOS probe cases).

- Info and warning messages: kdialog --msgbox / --sorry, zenity --info /
  --warning, display alert (informational / warning / critical)
- Confirm (yes/no): kdialog --yesno, zenity --question,
  display dialog returning button returned. Custom button labels differ
  per backend and need care; a plain bool confirm is the clean form.
- Password: kdialog --password, zenity --entry --hide-text (or --password),
  display dialog with hidden answer
- Multiple choice: kdialog --checklist (use --separate-output), zenity
  --list --checklist --multiple, choose from list with multiple
  selections allowed. Output separators differ, and AppleScript list
  output is ambiguous for items containing commas; design the return
  type as Vec<String> and parse per backend rather than splitting one
  shared string.
- File open, single and multiple: kdialog --getopenfilename (--multiple),
  zenity --file-selection (--multiple), choose file
- File save: kdialog --getsavefilename, zenity --file-selection --save,
  choose file name
- Folder pick: kdialog --getexistingdirectory, zenity --file-selection
  --directory, choose folder
- File filters: kdialog filter string, zenity --file-filter, choose file
  of type. Formats do not translate literally; accept extensions and
  build each backend's form from them.
- Color: kdialog --getcolor, zenity --color-selection, choose color.
  Return formats differ (hex string vs RGB values); normalize to a
  simple RGB struct or a hex string.

## Tier 2: Linux only (kdialog and zenity, no osascript equivalent)

Gate these behind a capability check or document them as returning
Error::Unsupported on macOS. Do not fake them with display dialog.

- Date: kdialog --calendar, zenity --calendar. Note both return locale
  formatted dates by default; zenity accepts --date-format, kdialog
  does not, so parsing needs care.
- Slider: kdialog --slider, zenity --scale
- Multiline text, view and edit: kdialog --textbox / --textinputbox,
  zenity --text-info --editable

## Needs its own design, not another Request variant

- Progress: zenity --progress is driven by stdin lines, kdialog
  --progressbar returns a D-Bus object you update with separate calls,
  and osascript has no progress dialog. The current shape (spawn, block,
  read stdout once) cannot express either Linux form. Treat progress as
  a handle-based API if it is added at all.
- Notification: kdialog --passivepopup, zenity --notification, and
  display notification look alike but behave differently (zenity's is a
  tray icon style API). The tracker already uses notify-rust for real
  notifications; keep this out unless a clear use appears.

## Out of scope for the common API

Single-backend extras with no honest cross-backend form:

- kdialog: --geticon, --dontagain, --detailederror details field
- zenity: --forms (several fields in one window, pipe-separated output)
- osascript: choose application, choose URL

## Windows (in scope for 1.0 since Oct 30, 2026)

The first assessment (Oct 29) deferred Windows: no bundled
zenity/kdialog equivalent, and the PowerShell InputBox cannot
distinguish cancel from empty input. The Oct 30 reevaluation found
that verdict rested on the wrong tool. powershell.exe (Windows
PowerShell 5.1) is bundled with Windows, and through
Add-Type -AssemblyName System.Windows.Forms it reaches the real
Win32 common dialogs and arbitrary custom forms. Every result
comes back as a DialogResult, so cancel is distinguishable
everywhere, including a custom text form (Cancel versus OK with an
empty field are different outcomes). Every Tier 1 kind maps:
MessageBox for messages and confirm, a small custom form for text,
password (UseSystemPasswordChar), single choice, and multiple
choice (CheckedListBox), and the purpose-built OpenFileDialog,
SaveFileDialog, FolderBrowserDialog, and ColorDialog classes, with
filters built from extensions like every other backend.

Design rules, mirroring the osascript backend: fixed scripts, user
text passed as script arguments (a param block), never
interpolated; invoke as powershell.exe -NoProfile -STA
-ExecutionPolicy Bypass; set the console output encoding to UTF-8
inside the script; guard stdout hygiene (any stray pipeline output
pollutes the answer). Known costs: the largest scripts of any
backend, and Add-Type is blocked under Constrained Language Mode
on locked-down machines, where the backend fails honestly rather
than faking a result. Testing is the easy case for once: the
maintainer's machine dual-boots Windows, so this backend gets a
thorough in-person test. Whether the Tier 2 kinds (WinForms has a
date picker, a track bar, and multiline text) join later is a
post-1.0 question; Tier 2 stays Linux-only for 1.0.
