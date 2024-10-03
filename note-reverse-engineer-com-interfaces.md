# Reverse engineering COM interfaces required for Virtual Desktop manipulation

To manipulate virtual desktops in Windows it is required to use unstable COM interface that might change between Windows versions. This document therefore describes the process used to support new Windows versions.

[Issue #14 ("Reverse engineering process") on the C# library Slion/VirtualDesktop](https://github.com/Slion/VirtualDesktop/issues/14) describes how they go about finding the definition for the relevant COM interfaces:

> We need to document the reverse engineering process to make it easier to support future versions of Windows.
>
> My understanding is that you need to run a Python script from a fork of [GetVirtualDesktopAPI_DIA] that dumps the GUIDs and interfaces definitions from `twinui.pcshell.dll` using [Debug Interface Access] and [Microsoft Symbol Server].
>
> It would be nice to port that Python script to C# and integrate it to this repository. That would make it even easier to perform reverse engineering.

[GetVirtualDesktopAPI_DIA]: https://github.com/mzomparelli/GetVirtualDesktopAPI_DIA
[Debug Interface Access]: https://learn.microsoft.com/en-us/visualstudio/debugger/debug-interface-access/debug-interface-access-sdk
[Microsoft Symbol Server]: https://learn.microsoft.com/en-us/windows-hardware/drivers/debugger/microsoft-public-symbols

Another relevant source is the [Readme for the Slion/VirtualDesktop C# library](https://github.com/Slion/VirtualDesktop/blob/7e37b9848aef681713224dae558d2e51960cf41e/README.md#windows-version-support):

> ### Windows version support
>
> The class IDs of some of the undocumented interfaces we use tend to change a lot between different versions of Windows.
> If the demo application crashes on start-up chances are all you need to do is provide the proper IDs for the version of Windows you are running on.
>
> Open `regedit` and export this path into a file: `\HKEY_LOCAL_MACHINE\SOFTWARE\Classes\Interface`.
> Open the resulting reg file and search it for matches against the whole word of each interface name we need:
>
> - `IApplicationView`
> - `IApplicationViewCollection`
> - `IObjectArray`
> - `IServiceProvider`
> - `IVirtualDesktop`
> - `IVirtualDesktopManager`
> - `IVirtualDesktopManagerInternal`
> - `IVirtualDesktopNotification`
> - `IVirtualDesktopNotificationService`
> - `IVirtualDesktopPinnedApps`
>
> Once you have the IDs add them in a new `setting` element in [app.config].
> Make sure to specify the correct 5 digits Windows build version.
> You can get it using one of those methods:
>
> - From the UI run: `winver`
> - From shell run: `ver`
> - From powershell run: `cmd /c ver`
>
> Make sure to contribute back your changes.
>
> [app.config]: https://github.com/Slion/VirtualDesktop/blob/7e37b9848aef681713224dae558d2e51960cf41e/src/VirtualDesktop/app.config

It can also be worth looking at similar virtual desktop libraries to see if they already support the Windows version in question:

- [Slion/VirtualDesktop](https://github.com/Slion/VirtualDesktop): C# wrapper for the Virtual Desktop API on Windows 11.
  - Interface ids in [src/VirtualDesktop/app.config](https://github.com/Slion/VirtualDesktop/blob/main/src/VirtualDesktop/app.config)
  - Method signatures in [src/VirtualDesktop/Interop](https://github.com/Slion/VirtualDesktop/tree/main/src/VirtualDesktop/Interop)
    - These are actually compiled when the app is executed by the `ComInterfaceAssemblyBuilder.CreateAssembly` method at: [src/VirtualDesktop/Interop/ComInterfaceAssemblyBuilder.cs](https://github.com/Slion/VirtualDesktop/blob/main/src/VirtualDesktop/Interop/ComInterfaceAssemblyBuilder.cs)

- [MScholtes/VirtualDesktop](https://github.com/MScholtes/VirtualDesktop): C# command line tool to manage virtual desktops in Windows 10
  