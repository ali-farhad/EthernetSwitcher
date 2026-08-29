# Ethernet Switcher

A small Windows app for quickly changing between physical Ethernet adapters.

![Ethernet Switcher](assets/ethernet-switcher.png)

## Download

Open the [Releases](https://github.com/ali-farhad/EthernetSwitcher/releases) page and download one of these files:

- `EthernetSwitcher-v1.0.0-Setup.exe` — normal Windows installer
- `EthernetSwitcher-v1.0.0.exe` — portable version, no installation
- `EthernetSwitcher-v1.0.0.msi` — MSI installer

Windows asks for administrator permission because changing network adapters requires it. The files are not code-signed yet, so SmartScreen may show a warning.

## Publish a release

Commit and push your changes, then create and push a version tag:

```powershell
git tag v1.0.0
git push origin v1.0.0
```

GitHub Actions builds the portable app and both installers. When the build finishes, they appear automatically under **Releases**.

Made with 💖 by [alifarhad](https://github.com/ali-farhad).
