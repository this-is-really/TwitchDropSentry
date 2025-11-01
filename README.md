# 🚀 Release Candidate (0.3.0)

> [!NOTE]
>
> ### 🚀 Release Candidate Notice (**0.3.0**)
>
> This is a **Release Candidate (0.3.0)**. The major feature set is **complete**, and all critical bugs found during the Beta phase have been fixed.
>
> Your main purpose in testing this version is to:
>
> * Find any remaining **minor bugs** and unexpected edge-cases.
> * Evaluate **usability** and overall user experience.
> * Test the application under **real-world conditions**.
>
> While this build is **practically ready for production**, it is still a pre-release and may contain issues that affect data or performance. **Do not use this version for critical production data.**
>
> **Thank you for your feedback!**
>

---

## What is this?

This is a command-line tool designed to automatically watch Twitch streams and claim Time-Based Drops for a selected game.

It runs in the background, finds eligible streams, simulates watch time by sending the necessary **GQL** events, and automatically claims drops as they become available.

### How it Works

1. Logs into your Twitch account (saves credentials to `data/save.json`).
2. Fetches active Drop Campaigns and **groups them by game** to ask you to select one.
3. Finds and prioritizes the **best eligible live stream** for that campaign.
4. Simulates "watching" that stream. **Note:** The underlying $\text{GQL}$ implementation is powered by [**twitch-gql-rs**](https://github.com/this-is-really/twitch-gql-rs).
5. Monitors your drop progress with a **real-time terminal progress bar**.
6. **Automatically claims** the drop once the required time is met, with robust retry logic.
7. Saves claimed drops to `data/cash.json` to avoid re-claiming.

## 💻 Available Binaries

Standard pre-compiled binaries are provided for common platforms.

* **Windows:** Executable for **x86_64** architecture (**.exe** file).
* **Linux:** **ELF** executable for **x86_64** architecture.

## 🐞 Found a Bug?

Bugs were common during the Alpha stage, but this **Release Candidate is significantly more stable**. All critical issues found during the Beta phase have been fixed, and any remaining problems should be minor.

If you still encounter *any* crashes, errors, or unexpected behavior, please **open an Issue** in this repository.

## :tada: Did you like the app?

Please consider rating this repository by clicking the star in the top-right corner of the page on GitHub (you need to be logged into your account). This gives me the motivation to keep developing this project.

![Star](https://i.ibb.co/3YkyqJQ8/2025-10-31-20-25.png)

## ❤️ Support the Developer

<div align="center">

[![DonationAlerts](https://www.donationalerts.com/img/brand/donationalerts.svg)](https://www.donationalerts.com/r/this_is_really)

Your support will accelerate development and help ensure the long-term maintenance of this project.
