use serde::{Deserialize, Serialize};
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn concise_error(stderr: &[u8], fallback: &str) -> String {
    String::from_utf8_lossy(stderr)
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or(fallback)
        .to_owned()
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EthernetAdapter {
    guid: String,
    name: String,
    description: String,
    status: String,
    link_speed: String,
    mac_address: String,
    ipv4_address: Option<String>,
}

#[cfg(windows)]
fn powershell(script: &str) -> Result<String, String> {
    let mut command = Command::new("powershell.exe");
    command
        .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command", script])
        .creation_flags(CREATE_NO_WINDOW);

    let output = command
        .output()
        .map_err(|error| format!("Could not start Windows PowerShell: {error}"))?;

    if !output.status.success() {
        let detail = concise_error(
            &output.stderr,
            "Windows could not complete the network operation.",
        );
        return Err(if detail.is_empty() {
            "Windows could not complete the network operation. Try running the app as administrator."
                .to_owned()
        } else {
            detail
        });
    }

    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_owned())
        .map_err(|_| "PowerShell returned text in an unexpected encoding.".to_owned())
}

#[tauri::command]
fn list_ethernet_adapters() -> Result<Vec<EthernetAdapter>, String> {
    #[cfg(not(windows))]
    return Err("Ethernet Switcher is available on Windows only.".to_owned());

    #[cfg(windows)]
    {
        let script = r#"
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$OutputEncoding = [Console]::OutputEncoding = [System.Text.UTF8Encoding]::new()
$items = @(Get-NetAdapter -Physical | Where-Object { $_.InterfaceType -eq 6 } | ForEach-Object {
  $adapter = $_
  $ip = Get-NetIPAddress -InterfaceIndex $adapter.InterfaceIndex -AddressFamily IPv4 -ErrorAction SilentlyContinue |
    Where-Object { $_.IPAddress -notlike '169.254.*' } | Select-Object -First 1
  [PSCustomObject]@{
    guid = $adapter.InterfaceGuid.ToString()
    name = $adapter.Name
    description = $adapter.InterfaceDescription
    status = $adapter.Status.ToString()
    linkSpeed = if ($adapter.LinkSpeed) { $adapter.LinkSpeed } else { '—' }
    macAddress = if ($adapter.MacAddress) { $adapter.MacAddress } else { '—' }
    ipv4Address = if ($ip) { $ip.IPAddress } else { $null }
  }
})
ConvertTo-Json -InputObject ([array]$items) -Compress
"#;

        let json = powershell(script)?;
        if json.is_empty() || json == "null" {
            return Ok(Vec::new());
        }
        serde_json::from_str(&json)
            .map_err(|error| format!("Could not read the adapter list: {error}"))
    }
}

#[tauri::command]
fn switch_adapter(adapter_guid: String) -> Result<(), String> {
    #[cfg(not(windows))]
    return Err("Ethernet Switcher is available on Windows only.".to_owned());

    #[cfg(windows)]
    {
        let guid = adapter_guid
            .parse::<uuid::Uuid>()
            .map_err(|_| "The selected adapter has an invalid identifier.".to_owned())?;

        let script = r#"
$ErrorActionPreference = 'Stop'
$targetGuid = [Guid]::Parse($env:ETHERNET_SWITCHER_TARGET)
$ethernet = @(Get-NetAdapter -Physical | Where-Object { $_.InterfaceType -eq 6 })
$target = $ethernet | Where-Object { [Guid]$_.InterfaceGuid -eq $targetGuid } | Select-Object -First 1
if (-not $target) { throw 'The selected Ethernet adapter is no longer available.' }
if ($target.Status -eq 'Disabled') { $target | Enable-NetAdapter -Confirm:$false }
$ethernet | Where-Object { [Guid]$_.InterfaceGuid -ne $targetGuid -and $_.Status -ne 'Disabled' } |
  Disable-NetAdapter -Confirm:$false
Start-Sleep -Milliseconds 350
$result = Get-NetAdapter -Physical | Where-Object { [Guid]$_.InterfaceGuid -eq $targetGuid } | Select-Object -First 1
if ($result.Status -eq 'Disabled') { throw 'Windows did not enable the selected adapter.' }
"#;

        let output = Command::new("powershell.exe")
            .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command", script])
            .env("ETHERNET_SWITCHER_TARGET", guid.hyphenated().to_string())
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|error| format!("Could not start Windows PowerShell: {error}"))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(concise_error(
                &output.stderr,
                "Windows rejected the switch. Administrator access is required.",
            ))
        }
    }
}

#[tauri::command]
fn open_author_page() -> Result<(), String> {
    #[cfg(not(windows))]
    return Err("This link is available on Windows only.".to_owned());

    #[cfg(windows)]
    {
        use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};
        use windows_sys::Win32::UI::Shell::ShellExecuteW;
        use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

        let verb: Vec<u16> = OsStr::new("open").encode_wide().chain(once(0)).collect();
        let url: Vec<u16> = OsStr::new("https://github.com/ali-farhad")
            .encode_wide()
            .chain(once(0))
            .collect();
        let result = unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                verb.as_ptr(),
                url.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                SW_SHOWNORMAL,
            )
        };
        if result as isize > 32 {
            Ok(())
        } else {
            Err("Windows could not open the GitHub profile.".to_owned())
        }
    }
}

#[cfg(windows)]
fn relaunch_elevated_if_needed() -> bool {
    use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};
    use windows_sys::Win32::UI::Shell::{IsUserAnAdmin, ShellExecuteW};
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    unsafe {
        if IsUserAnAdmin() != 0 {
            return false;
        }
    }

    let Ok(executable) = std::env::current_exe() else { return false };
    let verb: Vec<u16> = OsStr::new("runas").encode_wide().chain(once(0)).collect();
    let file: Vec<u16> = executable.as_os_str().encode_wide().chain(once(0)).collect();
    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(), verb.as_ptr(), file.as_ptr(), std::ptr::null(),
            std::ptr::null(), SW_SHOWNORMAL,
        )
    };
    result as isize > 32
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(windows)]
    if relaunch_elevated_if_needed() { return; }

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            list_ethernet_adapters,
            switch_adapter,
            open_author_page
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
