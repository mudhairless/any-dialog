# any-dialog error message, rendered as a MessageBox under Windows
# PowerShell. Values arrive through ANYDIALOG_* environment
# variables, never as part of the script text.
# Exit codes: 0 = dismissed, 2 = error (message printed on stderr).
# There is no cancel path: the box has a single OK button.
$ErrorActionPreference = 'Stop'
try {
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
    Add-Type -AssemblyName System.Windows.Forms
    [System.Windows.Forms.Application]::EnableVisualStyles()

    $null = [System.Windows.Forms.MessageBox]::Show(
        $env:ANYDIALOG_MESSAGE,
        $env:ANYDIALOG_TITLE,
        [System.Windows.Forms.MessageBoxButtons]::OK,
        [System.Windows.Forms.MessageBoxIcon]::Error)
    exit 0
} catch {
    [Console]::Error.WriteLine($_.Exception.Message)
    exit 2
}
