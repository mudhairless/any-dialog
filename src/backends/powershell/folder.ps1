# any-dialog folder picker, rendered with the WinForms
# FolderBrowserDialog under Windows PowerShell. Values arrive
# through ANYDIALOG_* environment variables: TITLE (shown as
# the dialog's description text; the browser dialog has no
# caption of its own) and START (the folder to open on, when
# it names an existing folder; empty otherwise).
# Exit codes: 0 = path printed on stdout, 1 = cancelled
# (nothing printed anywhere), 2 = error (message on stderr).
$ErrorActionPreference = 'Stop'
try {
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
    Add-Type -AssemblyName System.Windows.Forms
    [System.Windows.Forms.Application]::EnableVisualStyles()

    $dialog = New-Object System.Windows.Forms.FolderBrowserDialog
    $dialog.Description = $env:ANYDIALOG_TITLE
    $dialog.ShowNewFolderButton = $true
    # Same $null-for-empty rule as open.ps1: "no start" is
    # $null here, not ''.
    $startPath = $env:ANYDIALOG_START
    if (-not [string]::IsNullOrEmpty($startPath) -and (Test-Path $startPath -PathType Container)) {
        $dialog.SelectedPath = $startPath
    }

    if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
        [Console]::Out.WriteLine($dialog.SelectedPath)
        exit 0
    }
    exit 1
} catch {
    [Console]::Error.WriteLine($_.Exception.Message)
    exit 2
}
