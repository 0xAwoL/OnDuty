# OnDuty

## What it solves
In our club, we don't have magnetic or remotely controlled doors. This creates the constant need to manually ask in groupchat: "Is anyone there right now? Can someone open the door?"

OnDuty solves this by automatically detecting who is currently present, so others can see at a glance if the space is occupied and accessible.

## How it works
- Members claim their devices (phones, laptops, etc.) via a simple web frontend.
- The core system runs on an RPi, which passively monitors the local network for claimed devices.
- Presence detection is completely passive (using ARP table monitoring).

## TODO
- Rewrite `network_monitor` to run natively on ESP32 - Implement a command execution wrapper.
- Implement device claiming via dedicated discord chatbot.

## How to use
```bash
git clone <this-repo-url>
cd onduty
cargo install 
cp .env.example .env # Edit .env with your configuration 
cargo check
cargo run