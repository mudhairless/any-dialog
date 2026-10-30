# any-dialog multiple choice prompt, rendered with WinForms
# under Windows PowerShell. The label and title arrive through
# ANYDIALOG_* environment variables; the choices arrive as
# ANYDIALOG_CHOICE_0 .. ANYDIALOG_CHOICE_<n-1> with the count in
# ANYDIALOG_CHOICE_COUNT, and the prechecked items the same way
# as ANYDIALOG_CURRENT_0 .. with ANYDIALOG_CURRENT_COUNT.
# Exit codes: 0 = picks printed on stdout, one per line (nothing
# printed when none are checked), 1 = cancelled (nothing printed
# anywhere), 2 = error (message printed on stderr).
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
    $form.ClientSize = New-Object System.Drawing.Size(400, 224)

    $label = New-Object System.Windows.Forms.Label
    $label.Location = New-Object System.Drawing.Point(12, 12)
    $label.Size = New-Object System.Drawing.Size(376, 30)
    $label.Text = $env:ANYDIALOG_LABEL
    $null = $form.Controls.Add($label)

    $current = @()
    $currentCount = [int]$env:ANYDIALOG_CURRENT_COUNT
    for ($i = 0; $i -lt $currentCount; $i++) {
        $current += [Environment]::GetEnvironmentVariable("ANYDIALOG_CURRENT_$i")
    }

    $listBox = New-Object System.Windows.Forms.CheckedListBox
    $listBox.Location = New-Object System.Drawing.Point(12, 48)
    $listBox.Size = New-Object System.Drawing.Size(376, 124)
    $listBox.CheckOnClick = $true
    $count = [int]$env:ANYDIALOG_CHOICE_COUNT
    for ($i = 0; $i -lt $count; $i++) {
        $item = [Environment]::GetEnvironmentVariable("ANYDIALOG_CHOICE_$i")
        $index = $listBox.Items.Add($item)
        if ($current -contains $item) {
            $listBox.SetItemChecked($index, $true)
        }
    }
    $null = $form.Controls.Add($listBox)

    $okButton = New-Object System.Windows.Forms.Button
    $okButton.Location = New-Object System.Drawing.Point(232, 186)
    $okButton.Size = New-Object System.Drawing.Size(75, 26)
    $okButton.Text = 'OK'
    $okButton.DialogResult = [System.Windows.Forms.DialogResult]::OK
    $null = $form.Controls.Add($okButton)

    $cancelButton = New-Object System.Windows.Forms.Button
    $cancelButton.Location = New-Object System.Drawing.Point(313, 186)
    $cancelButton.Size = New-Object System.Drawing.Size(75, 26)
    $cancelButton.Text = 'Cancel'
    $cancelButton.DialogResult = [System.Windows.Forms.DialogResult]::Cancel
    $null = $form.Controls.Add($cancelButton)

    $form.AcceptButton = $okButton
    $form.CancelButton = $cancelButton
    $form.ActiveControl = $listBox

    if ($form.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
        foreach ($checked in $listBox.CheckedItems) {
            [Console]::Out.WriteLine([string]$checked)
        }
        exit 0
    }
    exit 1
} catch {
    [Console]::Error.WriteLine($_.Exception.Message)
    exit 2
}
