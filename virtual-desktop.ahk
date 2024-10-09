; AutoHotkey v2 script
SetWorkingDir(A_ScriptDir)

; Path to the DLL, relative to the script
VDA_PATH := A_ScriptDir . "\VirtualDesktopAccessor.dll"
hVirtualDesktopAccessor := DllCall("LoadLibrary", "Str", VDA_PATH, "Ptr")

; Declare procedure handles
GetDesktopCountProc := DllCall("GetProcAddress", "Ptr", hVirtualDesktopAccessor, "AStr", "GetDesktopCount", "Ptr")
GoToDesktopNumberProc := DllCall("GetProcAddress", "Ptr", hVirtualDesktopAccessor, "AStr", "GoToDesktopNumber", "Ptr")
GetCurrentDesktopNumberProc := DllCall("GetProcAddress", "Ptr", hVirtualDesktopAccessor, "AStr", "GetCurrentDesktopNumber", "Ptr")
MoveWindowToDesktopNumberProc := DllCall("GetProcAddress", "Ptr", hVirtualDesktopAccessor, "AStr", "MoveWindowToDesktopNumber", "Ptr")

; Array to store last focused windows for each desktop
lastFocusedWindows := ["", "", "", "", "", "", "", "", "", ""]

; Function to get desktop count
GetDesktopCount() {
    global GetDesktopCountProc
    count := DllCall(GetDesktopCountProc, "Int")
    return count
}

; Function to get current desktop number
GetCurrentDesktopNumber() {
    global GetCurrentDesktopNumberProc
    return DllCall(GetCurrentDesktopNumberProc, "Int")
}

; Save the currently focused window
SaveCurrentWindowForDesktop() {
    global lastFocusedWindows
    currentDesktop := GetCurrentDesktopNumber()
    activeHwnd := WinGetID("A")
    lastFocusedWindows[currentDesktop + 1] := activeHwnd
}

; Activate the last focused window for the given desktop
ActivateLastFocusedWindowForDesktop(desktopNumber) {
    global lastFocusedWindows
    hwnd := lastFocusedWindows[desktopNumber + 1]

    if (hwnd) {
        ; Check if the window still exists
        if (WinExist("ahk_id " hwnd)) {
            WinActivate("ahk_id " hwnd)
        } else {
            ; Handle case where the window doesn't exist
            OutputDebug("Last focused window for desktop " desktopNumber " no longer exists.")
            lastFocusedWindows[desktopNumber + 1] = "" ; Clear the reference
        }
    }
}

; Move to a specific desktop
GoToDesktopNumber(num) {
    global GoToDesktopNumberProc
    WinActivate("ahk_class Shell_TrayWnd")
    DllCall(GoToDesktopNumberProc, "Int", num, "Int")
    ActivateLastFocusedWindowForDesktop(num) ; Activate the last focused window
}

MoveCurrentWindowToDesktop(number) {
    global MoveWindowToDesktopNumberProc, GoToDesktopNumberProc
    activeHwnd := WinExist("A")
    DllCall(MoveWindowToDesktopNumberProc, "Ptr", activeHwnd, "Int", number, "Int")
    DllCall(GoToDesktopNumberProc, "Int", number, "Int")
}

; Override the existing desktop change functions
MoveOrGotoDesktopNumber(num) {
    global lastFocusedWindows
    if (GetKeyState("LButton")) {
        lastFocusedWindows[GetCurrentDesktopNumber() + 1] := ""
        MoveCurrentWindowToDesktop(num)
        lastFocusedWindows[num + 1] := WinExist("A")
    } else {
        SaveCurrentWindowForDesktop() ; Save the currently focused window before changing desktops
        GoToDesktopNumber(num)
    }
    return
}

; Remaining functions...
; (Omitted for brevity; you can keep the original functions from your script)

!1::MoveOrGotoDesktopNumber(0)
!2::MoveOrGotoDesktopNumber(1)
!3::MoveOrGotoDesktopNumber(2)
!4::MoveOrGotoDesktopNumber(3)
!5::MoveOrGotoDesktopNumber(4)
!6::MoveOrGotoDesktopNumber(5)
!7::MoveOrGotoDesktopNumber(6)
!8::MoveOrGotoDesktopNumber(7)
!9::MoveOrGotoDesktopNumber(8)
!0::MoveOrGotoDesktopNumber(9)

; Don't forget to add your other keybindings and logic...
