//! Windows 10 Version 1803 (April 2018 Update)
//!
//! From [Wikipedia](https://en.wikipedia.org/wiki/Windows_10,_version_1803):
//! > Windows 10 April 2018 Update (also known as version 1803 and codenamed
//! > "Redstone 4") is the fifth major update to Windows 10 and the fourth in a
//! > series of updates under the Redstone codenames. It carries the build
//! > number 10.0.17134.
//!
//! # Interface definitions
//!
//! Some methods were noted as removed in this Windows version by
//! [`Win10Desktops.h` from
//! Ciantic/VirtualDesktopAccessor](https://github.com/Ciantic/VirtualDesktopAccessor/blob/5bc1bbaab247b5d72e70abc9432a15275fd2d229/VirtualDesktopAccessor/Win10Desktops.h).
//! See relevant commits:
//! - [Fixed the IApplicationView for 1803 · Ciantic/VirtualDesktopAccessor@2a8c3df](https://github.com/Ciantic/VirtualDesktopAccessor/commit/2a8c3dfca15d693b690b6c67d6b7eb57ee10934c)


use super::*;
use build_16299 as build_prev;

// These interfaces haven't changed since the previous version:
build_prev::IApplicationViewCollection!("2C08ADF0-A386-4B35-9250-0FE183476FCC");
build_prev::IVirtualDesktop!("FF72FFDD-BE7E-43FC-9C03-AD81681E88E4");
build_prev::IVirtualDesktopManagerInternal!("F31574D6-B682-4CDC-BD56-1827860ABEC6");
build_prev::IVirtualDesktopNotification!("C179334C-4295-40D3-BEA1-C654D965605A");
build_prev::IVirtualDesktopNotificationService!("0CD45E71-D927-4F15-8B0A-8FEF525337BF");
build_prev::IVirtualDesktopPinnedApps!("4CE81583-1E4C-4632-A621-07A53543148F");

// But these interfaces have different methods:

reusable_com_interface!(
    MacroOptions {
        temp_macro_name: _IApplicationView,
        iid: "871F602A-2B58-42B4-8C4B-6C43D642C06F", // Different from previous version
    },
    {
        pub unsafe trait IApplicationView: IUnknown {
            /* IInspecateble */
            pub unsafe fn get_iids(
                &self,
                out_iid_count: *mut ULONG,
                out_opt_iid_array_ptr: *mut *mut GUID,
            ) -> HRESULT;
            pub unsafe fn get_runtime_class_name(
                &self,
                out_opt_class_name: *mut HSTRING,
            ) -> HRESULT;
            pub unsafe fn get_trust_level(&self, ptr_trust_level: LPVOID) -> HRESULT;

            /* IApplicationView methods */
            pub unsafe fn set_focus(&self) -> HRESULT;
            pub unsafe fn switch_to(&self) -> HRESULT;

            pub unsafe fn try_invoke_back(&self, ptr_async_callback: IAsyncCallback) -> HRESULT;
            pub unsafe fn get_thumbnail_window(&self, out_hwnd: *mut HWND) -> HRESULT;
            pub unsafe fn get_monitor(&self, out_monitors: *mut *mut IImmersiveMonitor) -> HRESULT;
            pub unsafe fn get_visibility(&self, out_int: LPVOID) -> HRESULT;
            pub unsafe fn set_cloak(
                &self,
                application_view_cloak_type: APPLICATION_VIEW_CLOAK_TYPE,
                unknown: INT,
            ) -> HRESULT;
            pub unsafe fn get_position(
                &self,
                unknowniid: *const GUID,
                unknown_array_ptr: LPVOID,
            ) -> HRESULT;
            pub unsafe fn set_position(
                &self,
                view_position: *mut IApplicationViewPosition,
            ) -> HRESULT;
            pub unsafe fn insert_after_window(&self, window: HWND) -> HRESULT;
            pub unsafe fn get_extended_frame_position(&self, rect: *mut RECT) -> HRESULT;
            pub unsafe fn get_app_user_model_id(&self, id: *mut PWSTR) -> HRESULT; // Proc17
            pub unsafe fn set_app_user_model_id(&self, id: PCWSTR) -> HRESULT;
            pub unsafe fn is_equal_by_app_user_model_id(
                &self,
                id: PCWSTR,
                out_result: *mut INT,
            ) -> HRESULT;

            /*** IApplicationView methods ***/
            pub unsafe fn get_view_state(&self, out_state: *mut UINT) -> HRESULT; // Proc20
            pub unsafe fn set_view_state(&self, state: UINT) -> HRESULT; // Proc21
            pub unsafe fn get_neediness(&self, out_neediness: *mut INT) -> HRESULT; // Proc22
            pub unsafe fn get_last_activation_timestamp(
                &self,
                out_timestamp: *mut ULONGLONG,
            ) -> HRESULT;
            pub unsafe fn set_last_activation_timestamp(&self, timestamp: ULONGLONG) -> HRESULT;
            pub unsafe fn get_virtual_desktop_id(&self, out_desktop_guid: *mut GUID) -> HRESULT;
            pub unsafe fn set_virtual_desktop_id(&self, desktop_guid: *const GUID) -> HRESULT;
            pub unsafe fn get_show_in_switchers(&self, out_show: *mut INT) -> HRESULT;
            pub unsafe fn set_show_in_switchers(&self, show: INT) -> HRESULT;
            pub unsafe fn get_scale_factor(&self, out_scale_factor: *mut INT) -> HRESULT;
            pub unsafe fn can_receive_input(&self, out_can: *mut BOOL) -> HRESULT;
            pub unsafe fn get_compatibility_policy_type(
                &self,
                out_policy_type: *mut APPLICATION_VIEW_COMPATIBILITY_POLICY,
            ) -> HRESULT;
            pub unsafe fn set_compatibility_policy_type(
                &self,
                policy_type: APPLICATION_VIEW_COMPATIBILITY_POLICY,
            ) -> HRESULT;

            // Removed get_position_priority and set_position_priority that used
            // to be here, probably should have removed these in an earlier version

            pub unsafe fn get_size_constraints(
                &self,
                monitor: *mut IImmersiveMonitor,
                out_size1: *mut SIZE,
                out_size2: *mut SIZE,
            ) -> HRESULT;
            pub unsafe fn get_size_constraints_for_dpi(
                &self,
                dpi: UINT,
                out_size1: *mut SIZE,
                out_size2: *mut SIZE,
            ) -> HRESULT;
            pub unsafe fn set_size_constraints_for_dpi(
                &self,
                dpi: *const UINT,
                size1: *const SIZE,
                size2: *const SIZE,
            ) -> HRESULT;

            // Removed query_size_constraints_from_app that used to be here,
            // probably should have removed these in an earlier version

            pub unsafe fn on_min_size_preferences_updated(&self, window: HWND) -> HRESULT;
            pub unsafe fn apply_operation(
                &self,
                operation: *mut IApplicationViewOperation,
            ) -> HRESULT;
            pub unsafe fn is_tray(&self, out_is: *mut BOOL) -> HRESULT;
            pub unsafe fn is_in_high_zorder_band(&self, out_is: *mut BOOL) -> HRESULT;
            pub unsafe fn is_splash_screen_presented(&self, out_is: *mut BOOL) -> HRESULT;
            pub unsafe fn flash(&self) -> HRESULT;
            pub unsafe fn get_root_switchable_owner(
                &self,
                app_view: *mut IApplicationView,
            ) -> HRESULT; // proc45
            pub unsafe fn enumerate_ownership_tree(&self, objects: *mut IObjectArray) -> HRESULT; // proc46

            pub unsafe fn get_enterprise_id(&self, out_id: *mut PWSTR) -> HRESULT; // proc47
            pub unsafe fn is_mirrored(&self, out_is: *mut BOOL) -> HRESULT; //

            pub unsafe fn unknown1(&self, arg: *mut INT) -> HRESULT;
            pub unsafe fn unknown2(&self, arg: *mut INT) -> HRESULT;
            pub unsafe fn unknown3(&self, arg: *mut INT) -> HRESULT;
            pub unsafe fn unknown4(&self, arg: INT) -> HRESULT;
            pub unsafe fn unknown5(&self, arg: *mut INT) -> HRESULT;
            pub unsafe fn unknown6(&self, arg: INT) -> HRESULT;
            pub unsafe fn unknown7(&self) -> HRESULT;
            pub unsafe fn unknown8(&self, arg: *mut INT) -> HRESULT;
            pub unsafe fn unknown9(&self, arg: INT) -> HRESULT;
            pub unsafe fn unknown10(&self, arg: INT, arg2: INT) -> HRESULT;
            pub unsafe fn unknown11(&self, arg: INT) -> HRESULT;
            pub unsafe fn unknown12(&self, arg: *mut SIZE) -> HRESULT;
        }
    }
);
