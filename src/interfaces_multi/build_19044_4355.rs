//! Windows 10 21H2 Enterprise LTSC
//!
//! From
//! [Wikipedia](https://en.wikipedia.org/wiki/Windows_10_version_history#Version_22H2_(2022_Update)):
//! > The Windows 10 2022 Update (codenamed "22H2") is the thirteenth and final
//! > major update to Windows 10. It carries the build number 10.0.19045.
//!
//! # Interface definitions
//!
//! These definitions were found at [Feature: added windows 10 ltsc 19044.4355
//! COM object IDs by amadeo-alex · Pull Request #24 ·
//! Slion/VirtualDesktop](https://github.com/Slion/VirtualDesktop/pull/24).
//!
//! The interface ids are the same that are used by Windows 10 Version 22H2, so
//! it seems the LTSC edition changed their ids a bit early.

use super::*;

use build_17663 as build_prev;

// Same interface definitions as Windows 10, but different ids:
build_prev::IApplicationView!("372E1D3B-38D3-42E4-A15B-8AB2B178F513");
build_prev::IApplicationViewCollection!("1841C6D7-4F9D-42C0-AF41-8747538F10E5");
build_prev::IVirtualDesktop!("FF72FFDD-BE7E-43FC-9C03-AD81681E88E4");
build_prev::IVirtualDesktopManagerInternal!("F31574D6-B682-4CDC-BD56-1827860ABEC6");
build_prev::IVirtualDesktopNotification!("C179334C-4295-40D3-BEA1-C654D965605A");
build_prev::IVirtualDesktopNotificationService!("0CD45E71-D927-4F15-8B0A-8FEF525337BF");
build_prev::IVirtualDesktopPinnedApps!("4CE81583-1E4C-4632-A621-07A53543148F");
