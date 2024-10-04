//! Windows 11, version 24H2
//!
//! From [Wikipedia](https://en.wikipedia.org/wiki/Windows_11,_version_24H2):
//!
//! > The Windows 11 2024 Update (also known as version 24H2) is the third and
//! > current major update to Windows 11. It carries the build number
//! > 10.0.26100.
//!
//! # Interface definitions
//!
//! The interface definitions were found at
//! [MScholtes/VirtualDesktop/VirtualDesktop11-24H2.cs](https://github.com/MScholtes/VirtualDesktop/blob/c601d38796e947d7647c9124a5087fb4b595cbd9/VirtualDesktop11-24H2.cs).
//!
//! All the interface ids seem to have remained the same as in the previous
//! version but some methods were added to the interfaces.

use super::*;
use build_22621_2215 as prev_build;

// These interfaces haven't changed since the previous version:
prev_build::IVirtualDesktop!("3F07F4BE-B107-441A-AF0F-39D82529072C"); // Same IID
prev_build::IVirtualDesktopNotification!("B9E5E94D-233E-49AB-AF5C-2B4541C3AADE"); // Not used by MScholtes/VirtualDesktop, so don't know if had changes.
prev_build::IVirtualDesktopNotificationService!("0cd45e71-d927-4f15-8b0a-8fef525337bf"); // Not used by MScholtes/VirtualDesktop, so don't know if had changes.
prev_build::IVirtualDesktopPinnedApps!("4CE81583-1E4C-4632-A621-07A53543148F"); // Same IID

// But these interfaces have different methods:

reusable_com_interface!(
    MacroOptions {
        temp_macro_name: _IApplicationView,
        iid: "372E1D3B-38D3-42E4-A15B-8AB2B178F513", // Same as previous version
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

reusable_com_interface!(
    MacroOptions {
        temp_macro_name: _IApplicationViewCollection,
        iid: "1841C6D7-4F9D-42C0-AF41-8747538F10E5", // Same as previous version
    },
    {
        pub unsafe trait IApplicationViewCollection: IUnknown {
            pub unsafe fn get_views(&self, out_views: *mut IObjectArray) -> HRESULT;

            pub unsafe fn get_views_by_zorder(&self, out_views: *mut IObjectArray) -> HRESULT;

            pub unsafe fn get_views_by_app_user_model_id(
                &self,
                id: PCWSTR,
                out_views: *mut IObjectArray,
            ) -> HRESULT;

            pub unsafe fn get_view_for_hwnd(
                &self,
                window: HWND,
                out_view: *mut Option<IApplicationView>,
            ) -> HRESULT;

            pub unsafe fn get_view_for_application(
                &self,
                app: IImmersiveApplication,
                out_view: *mut IApplicationView,
            ) -> HRESULT;

            pub unsafe fn get_view_for_app_user_model_id(
                &self,
                id: PCWSTR,
                out_view: *mut IApplicationView,
            ) -> HRESULT;

            pub unsafe fn get_view_in_focus(&self, out_view: *mut IApplicationView) -> HRESULT;

            // This method is new (probably several versions ago):
            pub unsafe fn try_get_last_active_visible_view(
                &self,
                out_view: *mut IApplicationView,
            ) -> HRESULT;

            pub unsafe fn refresh_collection(&self) -> HRESULT;

            pub unsafe fn register_for_application_view_changes(
                &self,
                listener: IApplicationViewChangeListener,
                out_id: *mut DWORD,
            ) -> HRESULT;

            // Note: removed "register_for_application_view_position_changes"
            // that used to be here. Probably should have removed this in an
            // earlier version.

            pub unsafe fn unregister_for_application_view_changes(&self, id: DWORD) -> HRESULT;
        }
    }
);

reusable_com_interface!(
    MacroOptions {
        temp_macro_name: _IVirtualDesktopManagerInternal,
        iid: "53F5CA0B-158F-4124-900C-057158060B27", // Same as previous version
    },
    {
        pub unsafe trait IVirtualDesktopManagerInternal: IUnknown {
            pub unsafe fn get_desktop_count(&self, out_count: *mut UINT) -> HRESULT;

            pub unsafe fn move_view_to_desktop(
                &self,
                view: ComIn<IApplicationView>,
                desktop: ComIn<IVirtualDesktop>,
            ) -> HRESULT;

            pub unsafe fn can_move_view_between_desktops(
                &self,
                view: ComIn<IApplicationView>,
                can_move: *mut i32,
            ) -> HRESULT;

            pub unsafe fn get_current_desktop(
                &self,
                out_desktop: *mut Option<IVirtualDesktop>,
            ) -> HRESULT;

            pub unsafe fn get_desktops(&self, out_desktops: *mut Option<IObjectArray>) -> HRESULT;

            /// Get next or previous desktop
            ///
            /// Direction values:
            /// 3 = Left direction
            /// 4 = Right direction
            pub unsafe fn get_adjacent_desktop(
                &self,
                in_desktop: ComIn<IVirtualDesktop>,
                direction: UINT,
                out_pp_desktop: *mut Option<IVirtualDesktop>,
            ) -> HRESULT;

            pub unsafe fn switch_desktop(&self, desktop: ComIn<IVirtualDesktop>) -> HRESULT;

            // This method is new for build 26100:
            pub unsafe fn switch_desktop_and_move_foreground_view(
                &self,
                desktop: ComIn<IVirtualDesktop>,
            ) -> HRESULT;

            pub unsafe fn create_desktop(
                &self,
                out_desktop: *mut Option<IVirtualDesktop>,
            ) -> HRESULT;

            pub unsafe fn move_desktop(
                &self,
                in_desktop: ComIn<IVirtualDesktop>,
                index: UINT,
            ) -> HRESULT;

            pub unsafe fn remove_desktop(
                &self,
                destroy_desktop: ComIn<IVirtualDesktop>,
                fallback_desktop: ComIn<IVirtualDesktop>,
            ) -> HRESULT;

            pub unsafe fn find_desktop(
                &self,
                guid: *const GUID,
                out_desktop: *mut Option<IVirtualDesktop>,
            ) -> HRESULT;

            pub unsafe fn get_desktop_switch_include_exclude_views(
                &self,
                desktop: ComIn<IVirtualDesktop>,
                out_pp_desktops1: *mut IObjectArray,
                out_pp_desktops2: *mut IObjectArray,
            ) -> HRESULT;

            pub unsafe fn set_name(
                &self,
                desktop: ComIn<IVirtualDesktop>,
                name: HSTRING,
            ) -> HRESULT;
            pub unsafe fn set_wallpaper(
                &self,
                desktop: ComIn<IVirtualDesktop>,
                name: HSTRING,
            ) -> HRESULT;
            pub unsafe fn update_wallpaper_for_all(&self, name: HSTRING) -> HRESULT;

            pub unsafe fn copy_desktop_state(
                &self,
                view0: ComIn<IApplicationView>,
                view1: ComIn<IApplicationView>,
            ) -> HRESULT;

            pub unsafe fn create_remote_desktop(
                &self,
                name: HSTRING,
                out_desktop: *mut Option<IVirtualDesktop>,
            ) -> HRESULT;

            pub unsafe fn switch_remote_desktop(&self, desktop: ComIn<IVirtualDesktop>) -> HRESULT;

            pub unsafe fn switch_desktop_with_animation(
                &self,
                desktop: ComIn<IVirtualDesktop>,
            ) -> HRESULT;

            pub unsafe fn get_last_active_desktop(
                &self,
                out_desktop: *mut Option<IVirtualDesktop>,
            ) -> HRESULT;

            pub unsafe fn wait_for_animation_to_complete(&self) -> HRESULT;
        }
    }
);
