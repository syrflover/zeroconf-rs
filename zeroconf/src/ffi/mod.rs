//! Utilities related to FFI bindings

use libc::c_void;
use std::ptr;

pub(crate) mod c_str;

/// Helper trait to convert a raw `*mut c_void` to it's rust type
pub trait FromRaw<T> {
    /// Converts the specified `*mut c_void` to a `&'a mut T`.
    ///
    /// # Safety
    /// This function is unsafe due to the dereference of the specified raw pointer.
    unsafe fn from_raw<'a>(raw: *mut c_void) -> &'a mut T {
        assert_not_null!(raw);
        &mut *(raw as *mut T)
    }
}

/// Helper trait to convert and clone a raw `*mut c_void` to it's rust type
pub trait CloneRaw<T: FromRaw<T> + Clone> {
    /// Converts and clones the specified `*mut c_void` to a `Box<T>`.
    ///
    /// # Safety
    /// This function is unsafe due to a call to the unsafe function [`FromRaw::from_raw()`].
    ///
    /// [`FromRaw::from_raw()`]: trait.FromRaw.html#method.from_raw
    unsafe fn clone_raw(raw: *mut c_void) -> Box<T> {
        assert_not_null!(raw);
        Box::new(T::from_raw(raw).clone())
    }
}

/// Helper trait to convert self to a raw `*mut c_void`
pub trait AsRaw {
    /// Converts self to a raw `*mut c_void` by cast.
    fn as_raw(&mut self) -> *mut c_void {
        self as *mut _ as *mut c_void
    }
}

/// Helper trait to unwrap a type to a `*const T` or a null-pointer if not present.
pub trait UnwrapOrNull<T> {
    /// Unwraps this type to `*const T` or `ptr::null()` if not present.
    fn unwrap_or_null(&self) -> *const T;
}

impl<T> UnwrapOrNull<T> for Option<*const T> {
    fn unwrap_or_null(&self) -> *const T {
        self.unwrap_or_else(|| ptr::null() as *const T)
    }
}

/// Helper trait to unwrap a type to a `*mut T` or a null-pointer if not present.
pub trait UnwrapMutOrNull<T> {
    /// Unwraps this type to `*mut T` or `ptr::null_mut()` if not present.
    fn unwrap_mut_or_null(&mut self) -> *mut T;
}

impl<T> UnwrapMutOrNull<T> for Option<*mut T> {
    fn unwrap_mut_or_null(&mut self) -> *mut T {
        self.unwrap_or_else(|| ptr::null_mut() as *mut T)
    }
}

#[cfg(target_vendor = "apple")]
pub(crate) mod macos {
    use crate::Result;
    use libc::{fd_set, suseconds_t, time_t, timeval};
    use std::time::Duration;
    use std::{mem, ptr};

    /// Performs a unix `select()` on the specified `sock_fd` and `timeout`. Returns the select result
    /// or `Err` if the result is negative.
    ///
    /// # Safety
    /// This function is unsafe because it directly interfaces with C-library system calls.
    pub unsafe fn read_select(sock_fd: i32, timeout: Duration) -> Result<u32> {
        let mut read_flags: fd_set = mem::zeroed();

        libc::FD_ZERO(&mut read_flags);
        libc::FD_SET(sock_fd, &mut read_flags);

        let tv_sec = timeout.as_secs() as time_t;
        let tv_usec = timeout.subsec_micros() as suseconds_t;
        let mut timeout = timeval { tv_sec, tv_usec };

        let result = libc::select(
            sock_fd + 1,
            &mut read_flags,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut timeout,
        );

        if result < 0 {
            Err("select(): returned error status".into())
        } else {
            Ok(result as u32)
        }
    }
}

#[cfg(target_os = "windows")]
pub(crate) mod windows {
    use crate::Result;
    use std::time::Duration;
    use std::{mem, ptr};
    use windows_sys::Win32::Networking::WinSock as winsock;

    /*

    #define FD_SET(fd, set) do {
        u_int i;
        for (i = 0; i < ((fd_set FAR *)(set))->fd_count; i++) {
            if (((fd_set FAR *)(set))->fd_array[i] == (fd)) {
                break;
            }
        }
        if (i == ((fd_set FAR *)(set))->fd_count) {
            if (((fd_set FAR *)(set))->fd_count < FD_SETSIZE) {
                ((fd_set FAR *)(set))->fd_array[i] = (fd);
                ((fd_set FAR *)(set))->fd_count++;
            }
        }
    } while(0)

    #define FD_ZERO(set) (((fd_set FAR *)(set))->fd_count=0)

    */
    #[allow(non_snake_case)]
    fn FD_SET(fd: i32, set: &mut winsock::fd_set) {
        let mut i = 0;

        while i < set.fd_count {
            i += 1;

            if set.fd_array[i as usize] == fd as usize {
                break;
            }
        }

        if i == set.fd_count && set.fd_count < winsock::FD_SETSIZE {
            set.fd_array[i as usize] = fd as usize;
            set.fd_count += 1;
        }
    }

    #[allow(non_snake_case)]
    fn FD_ZERO(set: &mut winsock::fd_set) {
        set.fd_count = 0;
    }

    /// Performs a unix `select()` on the specified `sock_fd` and `timeout`. Returns the select result
    /// or `Err` if the result is negative.
    ///
    /// # Safety
    /// This function is unsafe because it directly interfaces with C-library system calls.
    pub unsafe fn read_select(sock_fd: i32, timeout: Duration) -> Result<u32> {
        let mut read_flags: winsock::fd_set = mem::zeroed();

        FD_ZERO(&mut read_flags);
        FD_SET(sock_fd, &mut read_flags);

        let tv_sec = timeout.as_secs() as i32;
        let tv_usec = timeout.subsec_micros() as i32;
        let timeout = winsock::timeval { tv_sec, tv_usec };

        let result = winsock::select(
            sock_fd + 1,
            &mut read_flags,
            ptr::null_mut(),
            ptr::null_mut(),
            &timeout,
        );

        if result < 0 {
            Err("select(): returned error status".into())
        } else {
            Ok(result as u32)
        }
    }
}
