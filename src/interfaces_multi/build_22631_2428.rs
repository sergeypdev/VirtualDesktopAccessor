//! Windows 11, version 23H2
//!
//! From [Wikipedia](https://en.wikipedia.org/wiki/Windows_11,_version_23H2):
//!
//! > The Windows 11 2023 Update (also known as version 23H2 and codenamed "Sun
//! > Valley 3") is the second major update to Windows 11. It was shipped as an
//! > enablement package for Windows 11 2022 Update and carries the build number
//! > 10.0.22631.
use super::*;
use build_22621_3155 as build_prev;

build_prev::IApplicationView!("372E1D3B-38D3-42E4-A15B-8AB2B178F513");
build_prev::IApplicationViewCollection!("1841C6D7-4F9D-42C0-AF41-8747538F10E5");
build_prev::IVirtualDesktop!("3F07F4BE-B107-441A-AF0F-39D82529072C");
build_prev::IVirtualDesktopManagerInternal!("A3175F2D-239C-4BD2-8AA0-EEBA8B0B138E"); // Changed
build_prev::IVirtualDesktopNotification!("B287FA1C-7771-471A-A2DF-9B6B21F0D675"); // Changed
build_prev::IVirtualDesktopNotificationService!("0cd45e71-d927-4f15-8b0a-8fef525337bf");
build_prev::IVirtualDesktopPinnedApps!("4CE81583-1E4C-4632-A621-07A53543148F");
