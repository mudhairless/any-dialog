# any-dialog color picker, rendered with the WinForms
# ColorDialog under Windows PowerShell. The common dialog has
# no caption, so only the initial color travels, as
# ANYDIALOG_INITIAL in #rrggbb form; the answer is printed in
# the same form.
# Exit codes: 0 = color printed on stdout, 1 = cancelled
# (nothing printed anywhere), 2 = error (message on stderr).
$ErrorActionPreference = 'Stop'
try {
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
    Add-Type -AssemblyName System.Windows.Forms
    Add-Type -AssemblyName System.Drawing
    [System.Windows.Forms.Application]::EnableVisualStyles()

    $dialog = New-Object System.Windows.Forms.ColorDialog
    $dialog.Color = [System.Drawing.ColorTranslator]::FromHtml($env:ANYDIALOG_INITIAL)

    if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
        $color = $dialog.Color
        [Console]::Out.WriteLine('#{0:x2}{1:x2}{2:x2}' -f $color.R, $color.G, $color.B)
        exit 0
    }
    exit 1
} catch {
    [Console]::Error.WriteLine($_.Exception.Message)
    exit 2
}
