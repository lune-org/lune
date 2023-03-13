<!-- markdownlint-disable MD033 -->
<!-- markdownlint-disable MD041 -->

<div align="center">

# Lune 🌙

[![Current Lune library version](https://img.shields.io/crates/v/lune.svg?label=Version)](https://crates.io/crates/lune)
[![CI status](https://github.com/filiptibell/lune/workflows/CI/badge.svg)](https://github.com/filiptibell/lune/actions)
[![Release status](https://shields.io/endpoint?url=https://badges.readysetplay.io/workflow/filiptibell/lune/release.yaml)](https://github.com/filiptibell/lune/actions)
[![Current Lune library version](https://img.shields.io/github/license/filiptibell/lune.svg?label=License&color=informational)](https://github.com/filiptibell/lune/blob/main/LICENSE.txt)

</div>

---

Lune is a standalone [Luau](https://luau-lang.org) script runtime meant to be an alternative to traditional shell scripts, with the goal of drastically simplifying the typical tasks shell scripts are used for, making them easier to read and maintain.

## Features {#features}

- A strictly minimal but powerful interface that is easy to read and remember, just like Lua itself
- Fully featured APIs for the filesystem, networking, stdio, all included in the small (~2mb) executable
- World-class documentation, on the web *or* directly in your editor, no network connection necessary
- A familiar scripting environment for Roblox developers, with an included 1-to-1 task scheduler port

## Non-goals {#non-goals}

- Making scripts short and terse - proper autocomplete / intellisense make scripting using Lune just as quick, and readability is important
- Running full Roblox game scripts outside of Roblox - there is some compatibility here already, but Lune is meant for different purposes

## Where do I start? {#starting-guide}

Head over to the [wiki](https://github.com/filiptibell/lune/wiki) to get started using Lune!
