# Lib-vmm

**Lib-vmm** is the official SDK for developing plugins for [Void Mod Manager (VMM)](https://github.com/void-mod-manager/app).

> [!WARNING]
> This project is currently under active development. Expect frequent changes and updates.

> lib-vmm (this package) is not associated with the [libvmm](https://github.com/libvmm/libvmm) package, the naming just happened to be a poor coincidence. We're sorry for any confusion in advanced, please make sure you double read your cargo.toml or install commands!

## Overview

Lib-vmm simplifies the creation of plugins for Void Mod Manager, specifically **Game Providers** and **Mod Providers**.
It provides a streamlined interface and set of tools to help you build plugins that can be easily integrated and loaded into VMM.

Even the official providers are built using this SDK. You can find their source code [here](https://github.com/void-mod-manager/providers)
> *(note: they may not be available yet, as we’re currently in the process of migrating them)*.
