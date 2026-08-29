# Ethernet Switcher

A small Windows desktop app for switching between physical Ethernet adapters. Selecting an adapter enables it and disables the other physical wired adapters.

## Download — no developer tools required

You do **not** need Rust, Node.js, Tauri, or any other development prerequisite on your PC.

1. Open this repository's **Actions** tab on GitHub.
2. Open the latest successful **Build Windows app** run.
3. Download the `Ethernet-Switcher-Windows` artifact.
4. Extract it and run `EthernetSwitcher.exe` directly. There is no installer.

Tagged versions (for example `v0.1.0`) are also attached to the repository's **Releases** page. Windows may display a SmartScreen warning because community builds are not code-signed.

The app asks for administrator access when opened because enabling and disabling network adapters is a privileged Windows operation. All adapter discovery and switching happens locally through built-in Windows PowerShell commands.

## Creating a build on GitHub

Push your changes, create a version tag, or run the workflow manually from the Actions tab. The GitHub-hosted Windows builder installs the toolchain and produces one portable `EthernetSwitcher.exe`. No build setup is required on the machine that downloads the result.

```powershell
git tag v0.1.0
git push origin v0.1.0
```

## Development

The UI uses Leptos compiled to WebAssembly and the native shell uses Tauri 2. Local development is optional; the GitHub workflow is the supported build path.
