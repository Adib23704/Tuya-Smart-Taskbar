<p align="center">
  <img src="src-tauri/icons/icon.ico" alt="Tuya Smart Taskbar" width="128" height="128">
</p>

<h1 align="center">Tuya Smart Taskbar</h1>

<p align="center">
  <strong>Control your Tuya smart home devices directly from the Windows taskbar</strong>
</p>

<p align="center">
  <a href="https://github.com/Adib23704/Tuya-Smart-Taskbar/releases/latest">
    <img src="https://img.shields.io/github/v/release/Adib23704/Tuya-Smart-Taskbar?style=flat-square" alt="Latest Release">
  </a>
  <a href="https://github.com/Adib23704/Tuya-Smart-Taskbar/blob/master/LICENSE">
    <img src="https://img.shields.io/github/license/Adib23704/Tuya-Smart-Taskbar?style=flat-square" alt="License">
  </a>
  <a href="https://github.com/Adib23704/Tuya-Smart-Taskbar/releases">
    <img src="https://img.shields.io/github/downloads/Adib23704/Tuya-Smart-Taskbar/total?style=flat-square" alt="Downloads">
  </a>
</p>

---

## About

Tuya Smart Taskbar is a lightweight Windows system tray application that lets you control your Tuya-compatible smart home devices without opening any app or browser. Built with [Tauri](https://tauri.app/) for minimal resource usage and maximum performance.

## Features

- **System Tray Integration** - Lives quietly in your taskbar, always one click away
- **Real-time Device Control** - Toggle switches, adjust fan speeds, set temperatures, and more
- **Auto-refresh** - Device states update automatically every 5 seconds
- **Auto-launch** - Optionally start with Windows
- **Lightweight** - Minimal memory footprint thanks to Tauri's Rust backend
- **Secure** - HMAC-SHA256 signed API requests, credentials stored locally
- **Theme Support** - Respects your Windows light/dark theme preference

## Supported Devices & Controls

| Control Type | Supported Operations |
|--------------|---------------------|
| **Switches & Lights** | On/Off toggle |
| **Smart Fans** | 5-level speed control |
| **Air Conditioners** | Temperature (16-30°C), Fan speed (4 levels), Mode (Auto/Cold/Dry/Wind) |
| **Smart Outlets** | On/Off toggle |

> Any Tuya device with standard status codes will work. The app dynamically builds menus based on each device's capabilities.

## Supported Regions

| Region | API Endpoint |
|--------|--------------|
| Central Europe | `openapi.tuyaeu.com` |
| Western Europe | `openapi-weaz.tuyaeu.com` |
| China | `openapi.tuyacn.com` |
| Western America | `openapi.tuyaus.com` |
| Eastern America | `openapi-ueaz.tuyaus.com` |
| India | `openapi.tuyain.com` |
| Singapore | `openapi-sg.iotbing.com` |

## Installation

### Download

Download the latest installer from the [Releases](https://github.com/Adib23704/Tuya-Smart-Taskbar/releases/latest) page.

### Build from Source

**Prerequisites:**
- [Node.js](https://nodejs.org/) (v18+)
- [pnpm](https://pnpm.io/) (v10+)
- [Rust](https://rustup.rs/) (stable)

```bash
# Clone the repository
git clone https://github.com/Adib23704/Tuya-Smart-Taskbar.git
cd Tuya-Smart-Taskbar

# Install dependencies
pnpm install

# Run in development mode
pnpm dev

# Build for production
pnpm build
```

The built executable will be in `src-tauri/target/release/`.

## Setup

### Getting Tuya API Credentials

1. **Create a Tuya Developer Account**
   - Go to [Tuya IoT Platform](https://iot.tuya.com/)
   - Sign up for a developer account

2. **Create a Cloud Project**
   - Navigate to **Cloud** → **Development** → **Create Cloud Project**
   - Select your data center region (must match your Tuya app region)
   - Choose **Smart Home** as the industry
   - Note down your **Access ID** and **Access Secret**

3. **Link Your Tuya App Account**
   - In your cloud project, go to **Devices** → **Link Tuya App Account**
   - Open the Tuya Smart or Smart Life app on your phone
   - Scan the QR code to link your account
   - Your devices will now appear in the cloud project

4. **Get Your User ID**
   - After linking, go to **Devices** → **All Devices**
   - Click on any device and find the **UID** (User ID) in the device details

### Configuring the App

1. Launch Tuya Smart Taskbar
2. Right-click the tray icon and select **Open Configuration**
3. Enter your credentials:
   - **Region**: Select the data center matching your Tuya app
   - **Access Key**: Your Access ID from the cloud project
   - **Secret Key**: Your Access Secret from the cloud project
   - **User ID**: The UID from your device details
4. Click **Test Connection** to verify
5. Click **Save** to apply

## Usage

Once configured, your devices will appear in the tray menu:

```
├─ Living Room Light
│  └─ Power [ON]
├─ Bedroom Fan
│  ├─ Power [OFF]
│  └─ Fan Speed → 1, 2, 3, 4, 5
├─ Air Conditioner
│  ├─ Power [ON]
│  ├─ Temperature → 16°C ... 30°C
│  ├─ Fan Speed → 1, 2, 3, 4
│  └─ Mode → Auto, Cold, Dry, Wind
├─ ──────────────
├─ Open Configuration
├─ About
├─ ──────────────
└─ Quit
```

Click any control to toggle or change its state. The menu updates automatically to reflect current device status.

## Tech Stack

- **Frontend**: Vanilla HTML/CSS/JavaScript
- **Backend**: Rust with Tauri v2
- **HTTP Client**: reqwest with async/await
- **Authentication**: HMAC-SHA256 request signing

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

**Zahin A. Adib**

- Website: [adibdev.me](https://adibdev.me/)
- GitHub: [@Adib23704](https://github.com/Adib23704)

---

<p align="center">
  Made with Rust and Tauri
</p>
