//! Support for Windows 11.
use super::*;

// These interfaces haven't changed since previous version:
build_10240::IApplicationView!("372E1D3B-38D3-42E4-A15B-8AB2B178F513");
build_10240::IApplicationViewCollection!("1841c6d7-4f9d-42c0-af41-8747538f10e5");

// But these interfaces have different methods:

reusable_com_interface!(
    MacroOptions {
        temp_macro_name: _IVirtualDesktop,
        iid: "3F07F4BE-B107-441A-AF0F-39D82529072C",
    },
    {
        pub unsafe trait IVirtualDesktop: IUnknown {
            pub unsafe fn is_view_visible(
                &self,
                p_view: ComIn<IApplicationView>,
                out_bool: *mut u32,
            ) -> HRESULT;
            pub unsafe fn get_id(&self, out_guid: *mut GUID) -> HRESULT;
            pub unsafe fn get_name(&self, out_string: *mut HSTRING) -> HRESULT;
            pub unsafe fn get_wallpaper(&self, out_string: *mut HSTRING) -> HRESULT;
        }
    }
);

reusable_com_interface!(
    MacroOptions {
        temp_macro_name: _IVirtualDesktopNotification,
        iid: "B9E5E94D-233E-49AB-AF5C-2B4541C3AADE",
    },
    {
        pub unsafe trait IVirtualDesktopNotification: IUnknown {
            pub unsafe fn virtual_desktop_created(
                &self,
                desktop: ComIn<IVirtualDesktop>,
            ) -> HRESULT;

            pub unsafe fn virtual_desktop_destroy_begin(
                &self,
                desktop_destroyed: ComIn<IVirtualDesktop>,
                desktop_fallback: ComIn<IVirtualDesktop>,
            ) -> HRESULT;

            pub unsafe fn virtual_desktop_destroy_failed(
                &self,
                desktop_destroyed: ComIn<IVirtualDesktop>,
                desktop_fallback: ComIn<IVirtualDesktop>,
            ) -> HRESULT;

            pub unsafe fn virtual_desktop_destroyed(
                &self,
                desktop_destroyed: ComIn<IVirtualDesktop>,
                desktop_fallback: ComIn<IVirtualDesktop>,
            ) -> HRESULT;

            pub unsafe fn proc7(&self) -> HRESULT;

            pub unsafe fn virtual_desktop_moved(
                &self,
                desktop: ComIn<IVirtualDesktop>,
                old_index: i64,
                new_index: i64,
            ) -> HRESULT;

            pub unsafe fn virtual_desktop_name_changed(
                &self,
                desktop: ComIn<IVirtualDesktop>,
                name: HSTRING,
            ) -> HRESULT;

            pub unsafe fn view_virtual_desktop_changed(
                &self,
                view: ComIn<IApplicationView>,
            ) -> HRESULT;

            pub unsafe fn current_virtual_desktop_changed(
                &self,
                desktop_old: ComIn<IVirtualDesktop>,
                desktop_new: ComIn<IVirtualDesktop>,
            ) -> HRESULT;

            pub unsafe fn virtual_desktop_wallpaper_changed(
                &self,
                desktop: ComIn<IVirtualDesktop>,
                name: HSTRING,
            ) -> HRESULT;

            pub unsafe fn virtual_desktop_switched(
                &self,
                desktop: ComIn<IVirtualDesktop>,
            ) -> HRESULT;

            pub unsafe fn remote_virtual_desktop_connected(
                &self,
                desktop: ComIn<IVirtualDesktop>,
            ) -> HRESULT;
        }

        /// Implements an unstable interface that is only valid for a single Windows
        /// version using a more stable trait that works for all Windows versions.
        #[windows::core::implement(IVirtualDesktopNotification)]
        pub struct VirtualDesktopNotificationAdaptor<T>
        where
            T: build_dyn::IVirtualDesktopNotification_Impl,
        {
            pub inner: T,
        }
        impl<T> IVirtualDesktopNotification_Impl for VirtualDesktopNotificationAdaptor<T>
        where
            T: build_dyn::IVirtualDesktopNotification_Impl,
        {
            unsafe fn current_virtual_desktop_changed(
                &self,
                desktop_old: ComIn<IVirtualDesktop>,
                desktop_new: ComIn<IVirtualDesktop>,
            ) -> HRESULT {
                self.inner
                    .current_virtual_desktop_changed(desktop_old.into(), desktop_new.into())
            }

            unsafe fn virtual_desktop_wallpaper_changed(
                &self,
                desktop: ComIn<IVirtualDesktop>,
                name: HSTRING,
            ) -> HRESULT {
                self.inner
                    .virtual_desktop_wallpaper_changed(desktop.into(), name)
            }

            unsafe fn virtual_desktop_created(&self, desktop: ComIn<IVirtualDesktop>) -> HRESULT {
                self.inner.virtual_desktop_created(desktop.into())
            }

            unsafe fn virtual_desktop_destroy_begin(
                &self,
                desktop_destroyed: ComIn<IVirtualDesktop>,
                desktop_fallback: ComIn<IVirtualDesktop>,
            ) -> HRESULT {
                self.inner.virtual_desktop_destroy_begin(
                    desktop_destroyed.into(),
                    desktop_fallback.into(),
                )
            }

            unsafe fn virtual_desktop_destroy_failed(
                &self,
                desktop_destroyed: ComIn<IVirtualDesktop>,
                desktop_fallback: ComIn<IVirtualDesktop>,
            ) -> HRESULT {
                self.inner.virtual_desktop_destroy_failed(
                    desktop_destroyed.into(),
                    desktop_fallback.into(),
                )
            }

            unsafe fn virtual_desktop_destroyed(
                &self,
                desktop_destroyed: ComIn<IVirtualDesktop>,
                desktop_fallback: ComIn<IVirtualDesktop>,
            ) -> HRESULT {
                self.inner.virtual_desktop_destroyed(
                    desktop_destroyed.into(),
                    desktop_fallback.into(),
                )
            }

            unsafe fn proc7(&self) -> HRESULT {
                HRESULT(0)
            }

            unsafe fn virtual_desktop_moved(
                &self,
                desktop: ComIn<IVirtualDesktop>,
                old_index: i64,
                new_index: i64,
            ) -> HRESULT {
                self.inner
                    .virtual_desktop_moved(desktop.into(), old_index, new_index)
            }

            unsafe fn virtual_desktop_name_changed(
                &self,
                desktop: ComIn<IVirtualDesktop>,
                name: HSTRING,
            ) -> HRESULT {
                self.inner
                    .virtual_desktop_name_changed(desktop.into(), name)
            }

            unsafe fn view_virtual_desktop_changed(
                &self,
                view: ComIn<IApplicationView>,
            ) -> HRESULT {
                self.inner.view_virtual_desktop_changed(view.into())
            }

            unsafe fn virtual_desktop_switched(&self, desktop: ComIn<IVirtualDesktop>) -> HRESULT {
                self.inner.virtual_desktop_switched(desktop.into())
            }

            unsafe fn remote_virtual_desktop_connected(
                &self,
                desktop: ComIn<IVirtualDesktop>,
            ) -> HRESULT {
                self.inner
                    .remote_virtual_desktop_connected(desktop.into())
            }
        }
    }
);

reusable_com_interface!(
    MacroOptions {
        temp_macro_name: _IVirtualDesktopNotificationService,
        iid: "0CD45E71-D927-4F15-8B0A-8FEF525337BF",
    },
    {
        pub unsafe trait IVirtualDesktopNotificationService: IUnknown {
            pub unsafe fn register(
                &self,
                notification: *mut std::ffi::c_void, // *const IVirtualDesktopNotification,
                out_cookie: *mut DWORD,
            ) -> HRESULT;

            pub unsafe fn unregister(&self, cookie: u32) -> HRESULT;
        }
    }
);

reusable_com_interface!(
    MacroOptions {
        temp_macro_name: _IVirtualDesktopManagerInternal,
        iid: "53F5CA0B-158F-4124-900C-057158060B27",
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
        }
    }
);

reusable_com_interface!(
    MacroOptions {
        temp_macro_name: _IVirtualDesktopPinnedApps,
        iid: "4CE81583-1E4C-4632-A621-07A53543148F",
    },
    {
        pub unsafe trait IVirtualDesktopPinnedApps: IUnknown {
            pub unsafe fn is_app_pinned(&self, app_id: PCWSTR, out_iss: *mut bool) -> HRESULT;
            pub unsafe fn pin_app(&self, app_id: PCWSTR) -> HRESULT;
            pub unsafe fn unpin_app(&self, app_id: PCWSTR) -> HRESULT;

            pub unsafe fn is_view_pinned(
                &self,
                view: ComIn<IApplicationView>,
                out_iss: *mut bool,
            ) -> HRESULT;
            pub unsafe fn pin_view(&self, view: ComIn<IApplicationView>) -> HRESULT;
            pub unsafe fn unpin_view(&self, view: ComIn<IApplicationView>) -> HRESULT;
        }
    }
);
