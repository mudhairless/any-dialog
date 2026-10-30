# any-dialog yes/no question, rendered as a MessageBox under
# Windows PowerShell. Values arrive through ANYDIALOG_*
# environment variables, never as part of the script text.
# The answer is printed on stdout as the token "yes" or "no".
# A YesNo box cannot be dismissed without answering (Windows
# disables its close button), so there is no cancel path.
# Exit codes: 0 = answered, 2 = error (message printed on stderr).
$ErrorActionPreference = 'Stop'
try {
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
    Add-Type -AssemblyName System.Windows.Forms
    [System.Windows.Forms.Application]::EnableVisualStyles()

    $result = [System.Windows.Forms.MessageBox]::Show(
        $env:ANYDIALOG_MESSAGE,
        $env:ANYDIALOG_TITLE,
        [System.Windows.Forms.MessageBoxButtons]::YesNo,
        [System.Windows.Forms.MessageBoxIcon]::Question)

    if ($result -eq [System.Windows.Forms.DialogResult]::Yes) {
        [Console]::Out.WriteLine('yes')
    } else {
        [Console]::Out.WriteLine('no')
    }
    exit 0
} catch {
    [Console]::Error.WriteLine($_.Exception.Message)
    exit 2
}
