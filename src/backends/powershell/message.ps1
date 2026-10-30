# any-dialog message, rendered as a MessageBox under Windows
# PowerShell. Values arrive through ANYDIALOG_* environment
# variables, never as part of the script text. ANYDIALOG_ICON
# carries the level name: Information, Warning, or Error.
# Exit codes: 0 = dismissed, 2 = error (message printed on stderr).
# There is no cancel path: the box has a single OK button.
$ErrorActionPreference = 'Stop'
try {
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
    Add-Type -AssemblyName System.Windows.Forms
    [System.Windows.Forms.Application]::EnableVisualStyles()

    $icon = [System.Windows.Forms.MessageBoxIcon]::Information
    if ($env:ANYDIALOG_ICON -eq 'Warning') {
        $icon = [System.Windows.Forms.MessageBoxIcon]::Warning
    }
    if ($env:ANYDIALOG_ICON -eq 'Error') {
        $icon = [System.Windows.Forms.MessageBoxIcon]::Error
    }

    $null = [System.Windows.Forms.MessageBox]::Show(
        $env:ANYDIALOG_MESSAGE,
        $env:ANYDIALOG_TITLE,
        [System.Windows.Forms.MessageBoxButtons]::OK,
        $icon)
    exit 0
} catch {
    [Console]::Error.WriteLine($_.Exception.Message)
    exit 2
}
