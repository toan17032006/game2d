# Bring the running game window to the foreground (restore if minimized).
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class Win {
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
}
'@ -ErrorAction SilentlyContinue

$p = Get-Process tiny-swords -ErrorAction SilentlyContinue | Select-Object -First 1
if ($p -and $p.MainWindowHandle -ne [IntPtr]::Zero) {
    [Win]::ShowWindow($p.MainWindowHandle, 9) | Out-Null   # SW_RESTORE
    [Win]::SetForegroundWindow($p.MainWindowHandle) | Out-Null
    Write-Output ("FOCUSED: " + $p.MainWindowTitle + " (PID " + $p.Id + ")")
} else {
    Write-Output "NO WINDOW"
}
