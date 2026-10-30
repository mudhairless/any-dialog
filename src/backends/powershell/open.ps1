# any-dialog open-file picker, rendered with the WinForms
# OpenFileDialog under Windows PowerShell. Values arrive
# through ANYDIALOG_* environment variables: TITLE, START
# (where the dialog opens; empty for the default place),
# FILTER (the complete WinForms filter string, built by the
# crate), and MULTIPLE ('1' allows several picks).
# Exit codes: 0 = pick printed on stdout, one per line,
# 1 = cancelled (nothing printed anywhere), 2 = error
# (message printed on stderr).
$ErrorActionPreference = 'Stop'
try {
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
    Add-Type -AssemblyName System.Windows.Forms
    [System.Windows.Forms.Application]::EnableVisualStyles()

    $dialog = New-Object System.Windows.Forms.OpenFileDialog
    $dialog.Title = $env:ANYDIALOG_TITLE
    $dialog.Filter = $env:ANYDIALOG_FILTER
    $dialog.Multiselect = $env:ANYDIALOG_MULTIPLE -eq '1'
    if ($env:ANYDIALOG_START -ne '') {
        if (Test-Path $env:ANYDIALOG_START -PathType Container) {
            $dialog.InitialDirectory = $env:ANYDIALOG_START
        } else {
            $dialog.InitialDirectory = [System.IO.Path]::GetDirectoryName($env:ANYDIALOG_START)
            $dialog.FileName = [System.IO.Path]::GetFileName($env:ANYDIALOG_START)
        }
    }

    if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
        if ($dialog.Multiselect) {
            foreach ($name in $dialog.FileNames) {
                [Console]::Out.WriteLine($name)
            }
        } else {
            [Console]::Out.WriteLine($dialog.FileName)
        }
        exit 0
    }
    exit 1
} catch {
    [Console]::Error.WriteLine($_.Exception.Message)
    exit 2
}
