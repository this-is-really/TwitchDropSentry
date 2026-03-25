# 🛡️ TwitchDropSentry

[![Discord](https://img.shields.io/discord/1437005378750775359?style=for-the-badge&logo=discord)](https://discord.gg/7H7n4RPtJG)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org)

**Automatically watches Twitch streams and farms Time-Based Drops for your chosen game.**

TwitchDropSentry is a lightweight, reliable Rust CLI tool that logs into your Twitch account and emulates stream viewing via the official Twitch GQL API — so you can collect drops completely hands-free.

---

## ✨ Key Features

- ✅ **Automatic login** — saves your session to `data/save.json`
- ✅ **Smart campaign selection** — groups active Twitch Drops by game
- ✅ **Priority streams** — finds and connects to the best live stream for the campaign
- ✅ **Real viewing simulation** — sends proper GQL events (powered by [**twitch-gql-rs**](https://github.com/this-is-really/twitch-gql-rs))
- ✅ **Live progress** — beautiful real-time terminal progress bar
- ✅ **Auto-claim** — instantly claims drops with robust retry logic
- ✅ **Progress tracking** — saves claimed drops to `data/cash.json`
- ✅ **Autostart** — launches with your OS via `settings.json`
- ✅ **Cross-platform** — ready-to-use binaries for Windows & Linux + Docker support

## 🚀 Quick Start

1. Download the latest release from [Releases](https://github.com/this-is-really/TwitchDropSentry/releases)
2. Run the executable
3. Log in to your Twitch account
4. Choose a game or configure `data/settings.json` for fully automatic operation

## 📥 Installation

### Prebuilt Binaries

| Platform      | File                     | Architecture |
|---------------|--------------------------|--------------|
| **Windows**   | `TwitchDropSentry.exe`   | x86_64       |
| **Linux**     | `TwitchDropSentry`       | x86_64 ELF   |

### Docker 🐳 (Community)

Maintained by [@Addison-Usc](https://github.com/Addison-Usc):

→ [Addison-Usc/TwitchDropBot](https://github.com/Addison-Usc/TwitchDropBot)

### Build from Source

```bash
git clone https://github.com/this-is-really/TwitchDropSentry.git
cd TwitchDropSentry
cargo build --release
```

(Rust toolchain required)

## ⚙️ Settings (data/settings.json)

Since version **0.3.5** you can fully automate the tool:

```json
{
  "game": "Rust",
  "autostart": true
}
```

- `"game"` — game name (empty string `""` = interactive selection)
- `"autostart": true` — start automatically on system boot

## 🔍 How It Works

1. Logs into Twitch and stores credentials in `data/save.json`
2. Fetches all active Drop campaigns via GQL and groups them by game
3. Finds the best eligible live stream
4. Simulates watching by sending GQL events
5. Shows real-time progress in the terminal
6. Automatically claims the drop when time is reached
7. Saves claimed drops to `data/cash.json`

## ⚠️ Security Notice

**Your credentials are stored in plain text** in `data/save.json` (username + token).  
- Use the tool **only on trusted devices**.  
- We strongly recommend creating a dedicated Twitch account just for farming drops.  
- The developer is not responsible for any account issues.

## 🐞 Found a Bug?

The project is in Release Candidate stage and very stable.  
If anything breaks, please open an [Issue](https://github.com/this-is-really/TwitchDropSentry/issues) with your OS, version, and logs.

## ⭐ Support the Project

If you like the tool, please give it a **star** on GitHub!  
It’s the best motivation to keep improving it.

## ❤️ Support the Developer

<div align="center">
  <a href="https://www.donationalerts.com/r/this_is_really">
    <img src="https://www.donationalerts.com/img/brand/donationalerts.svg" height="40">
  </a>
  <br><br>
  <a href="https://boosty.to/this-is-really">Boosty</a>
</div>

Your support helps bring new features faster and ensures long-term maintenance.

---

**Made with ❤️ for the Twitch community**  
**License:** [MIT](LICENSE)  
**Version:** 0.3.5 (Release Candidate 5)
