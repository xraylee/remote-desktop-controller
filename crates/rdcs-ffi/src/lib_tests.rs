// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Unit tests for FFI functions

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::ffi::CString;

    #[test]
    fn test_engine_create_and_destroy() {
        unsafe {
            let config = CString::new("{}").unwrap();
            let handle = rdcs_engine_create(config.as_ptr());
            assert!(!handle.is_null(), "Engine handle should not be null");
            rdcs_engine_destroy(handle);
        }
    }

    #[test]
    fn test_generate_invite_basic() {
        unsafe {
            // Create engine
            let config = CString::new("{}").unwrap();
            let handle = rdcs_engine_create(config.as_ptr());
            assert!(!handle.is_null(), "Engine handle should not be null");

            // Generate invite code
            let code_ptr = rdcs_generate_invite(handle);
            assert!(!code_ptr.is_null(), "Invite code pointer should not be null");

            // Convert to Rust string
            let code_cstr = std::ffi::CStr::from_ptr(code_ptr);
            let code_str = code_cstr.to_str().unwrap();

            println!("✅ Generated invite code: {}", code_str);

            // Verify format (4 digits)
            assert_eq!(code_str.len(), 4, "Invite code should be 4 characters");
            assert!(
                code_str.chars().all(|c| c.is_ascii_digit()),
                "Invite code should be all digits"
            );

            // Free string
            rdcs_free_string(code_ptr);

            // Generate another to test counter
            let code_ptr2 = rdcs_generate_invite(handle);
            assert!(!code_ptr2.is_null());

            let code_cstr2 = std::ffi::CStr::from_ptr(code_ptr2);
            let code_str2 = code_cstr2.to_str().unwrap();
            println!("✅ Second invite code: {}", code_str2);

            rdcs_free_string(code_ptr2);

            // Clean up
            rdcs_engine_destroy(handle);
        }
    }

    #[test]
    fn test_generate_invite_null_handle() {
        unsafe {
            let code_ptr = rdcs_generate_invite(std::ptr::null_mut());
            assert!(code_ptr.is_null(), "Should return null for null handle");
        }
    }

    #[test]
    fn test_cstring_conversion() {
        let test_str = "1234";
        let c_str = string_to_cstring(test_str);
        assert!(!c_str.is_null());

        unsafe {
            let result = std::ffi::CStr::from_ptr(c_str).to_str().unwrap();
            assert_eq!(result, test_str);
            rdcs_free_string(c_str);
        }
    }
}
