# any-dialog text prompt, rendered with WinForms under Windows
# PowerShell. Values arrive through ANYDIALOG_* environment
# variables, never as part of the script text.
# Exit codes: 0 = answer printed on stdout, 1 = cancelled (nothing
# printed anywhere), 2 = error (message printed on stderr).
$ErrorActionPreference = 'Stop'
try {
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
    Add-Type -AssemblyName System.Windows.Forms
    [System.Windows.Forms.Application]::EnableVisualStyles()

    $form = New-Object System.Windows.Forms.Form
    $form.Text = $env:ANYDIALOG_TITLE
    $form.StartPosition = [System.Windows.Forms.FormStartPosition]::CenterScreen
    $form.FormBorderStyle = [System.Windows.Forms.FormBorderStyle]::FixedDialog
    $form.MaximizeBox = $false
    $form.MinimizeBox = $false
    $form.TopMost = $true
    $form.ClientSize = New-Object System.Drawing.Size(400, 142)

    $label = New-Object System.Windows.Forms.Label
    $label.Location = New-Object System.Drawing.Point(12, 12)
    $label.Size = New-Object System.Drawing.Size(376, 30)
    $label.Text = $env:ANYDIALOG_LABEL
    $null = $form.Controls.Add($label)

    $inputBox = New-Object System.Windows.Forms.TextBox
    $inputBox.Location = New-Object System.Drawing.Point(12, 48)
    $inputBox.Size = New-Object System.Drawing.Size(376, 23)
    $inputBox.Text = $env:ANYDIALOG_INITIAL
    $null = $form.Controls.Add($inputBox)

    $okButton = New-Object System.Windows.Forms.Button
    $okButton.Location = New-Object System.Drawing.Point(232, 92)
    $okButton.Size = New-Object System.Drawing.Size(75, 26)
    $okButton.Text = 'OK'
    $okButton.DialogResult = [System.Windows.Forms.DialogResult]::OK
    $null = $form.Controls.Add($okButton)

    $cancelButton = New-Object System.Windows.Forms.Button
    $cancelButton.Location = New-Object System.Drawing.Point(313, 92)
    $cancelButton.Size = New-Object System.Drawing.Size(75, 26)
    $cancelButton.Text = 'Cancel'
    $cancelButton.DialogResult = [System.Windows.Forms.DialogResult]::Cancel
    $null = $form.Controls.Add($cancelButton)

    $form.AcceptButton = $okButton
    $form.CancelButton = $cancelButton
    $form.ActiveControl = $inputBox
    $inputBox.SelectAll()

    if ($form.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
        [Console]::Out.WriteLine($inputBox.Text)
        exit 0
    }
    exit 1
} catch {
    [Console]::Error.WriteLine($_.Exception.Message)
    exit 2
}
