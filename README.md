
# Drop_Sentry (Alpha)

> [!CAUTION]
> HEAVY ALPHA VERSION
>
> This is a very early, untested, pre-release version. It is **not stable**.
>
> The code is raw, likely full of bugs, and may not function as intended. It was uploaded to GitHub for version tracking, not for public use.
>
> **Please use at your own risk.**

## What is this?

This is a command-line tool designed to automatically watch Twitch streams and claim Time-Based Drops for a selected game.

It runs in the background, finds eligible streams, simulates watch time by sending the necessary GQL events, and automatically claims drops as they become available.

### How it Works (Theoretically)

1. Logs into your Twitch account (saves credentials to `data/save.json`).
2. Fetches active Drop Campaigns and asks you to select a game.
3. Finds an eligible live stream for that campaign.
4. Simulates "watching" that stream. **Note:** The underlying $\text{GQL}$ implementation is powered by [**twitch-gql-rs**](https://github.com/this-is-really/twitch-gql-rs)
5. Monitors your drop progress with a terminal progress bar.
6. Automatically claims the drop once the required time is met.
7. Saves claimed drops to `data/cash.json` to avoid re-claiming.

## 💻 Available Binaries

Standard pre-compiled binaries are provided for common platforms.

* **Windows:** Executable for **x86\_64** architecture (**$\text{.exe}$** file).
* **Linux:** **$\text{ELF}$** executable for **x86\_64** architecture.

## 🐞 Found a Bug? (You probably will!)

This project is in its earliest stages. If you decide to run this and encounter *any* crashes, errors, or unexpected behavior, please **open an Issue** in this repository.

All feedback, bug reports, and stack traces are incredibly helpful!
