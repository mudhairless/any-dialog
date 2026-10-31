# any-dialog save-file picker, rendered with the WinForms
# SaveFileDialog under Windows PowerShell. Values arrive
# through ANYDIALOG_* environment variables: TITLE, START (a
# suggested path; its folder opens and its name is prefilled,
# empty for neither), and FILTER (the complete WinForms filter
# string, built by the crate). The dialog's own overwrite
# prompt stays on, which is the native behaviour.
# Exit codes: 0 = path printed on stdout, 1 = cancelled
# (nothing printed anywhere), 2 = error (message on stderr).
$ErrorActionPreference = 'Stop'
try {
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
    Add-Type -AssemblyName System.Windows.Forms
    [System.Windows.Forms.Application]::EnableVisualStyles()

    $dialog = New-Object System.Windows.Forms.SaveFileDialog
    $dialog.Title = $env:ANYDIALOG_TITLE
    $dialog.Filter = $env:ANYDIALOG_FILTER
    # Same $null-for-empty rule as open.ps1: "no start"
    # arrives as $null through the $env: drive, not ''.
    $startPath = $env:ANYDIALOG_START
    if (-not [string]::IsNullOrEmpty($startPath)) {
        if (Test-Path $startPath -PathType Container) {
            $dialog.InitialDirectory = $startPath
        } else {
            $dialog.InitialDirectory = [System.IO.Path]::GetDirectoryName($startPath)
            $dialog.FileName = [System.IO.Path]::GetFileName($startPath)
        }
    }

    if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
        [Console]::Out.WriteLine($dialog.FileName)
        exit 0
    }
    exit 1
} catch {
    [Console]::Error.WriteLine($_.Exception.Message)
    exit 2
}
