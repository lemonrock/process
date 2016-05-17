// This file is part of process. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/process/master/COPYRIGHT. No part of process, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2016 The developers of process. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/process/master/COPYRIGHT.

#![feature(const_fn)]

#[macro_use] extern crate once;
extern crate libc;

use self::libc::getpid;
use self::libc::sysconf;
use self::libc::gethostname;
use self::libc::strnlen;
use self::libc::pid_t;
use self::libc::c_char;
use self::libc::size_t;
use self::libc::_SC_HOST_NAME_MAX;
use std::io::Error;
use std::io::ErrorKind;
use std::ffi::CStr;

lazy_static!
{
	pub static ref CurrentProcess: Process = new();
}

const UnknownHostName: &'static str = "(unknown)";

#[derive(Debug, Clone)]
pub struct Process
{
	pub hostName: String,
	pub hostNameWithoutDomain: String,
	pub programName: String,
	pub pid: pid_t
}

fn new() -> Process
{
	assert_has_not_been_called!("Only one instance of a Proces can exist");
	
	let hostName: String = match get_host_name_safely()
	{
		Err(_) => UnknownHostName.to_string(),
		Ok(hostName) => hostName
	};
	
	let hostNameWithoutDomain: String = match hostName.find('.')
	{
		None => hostName.clone(),
		Some(0) => String::new(),
		Some(index) => String::from(&hostName[..index])
	};
	
	let programName: String = get_program_name_safely();
		
	Process
	{
		hostName: hostName,
		hostNameWithoutDomain: hostNameWithoutDomain,
		programName: programName,
		pid: unsafe { getpid() },
	}
}

#[cfg(not(any(target_os="linux", target_os="android", target_os="windows")))]
pub fn get_program_name_safely() -> String
{
	unsafe { CStr::from_ptr(self::libc::getprogname()).to_string_lossy().into_owned() }
}

#[cfg(target_os="linux")]
pub fn get_program_name_safely() -> String
{
	unsafe { CStr::from_ptr(self::libc::program_invocation_short_name).to_string_lossy().into_owned() }
}

#[cfg(target_os="android")]
pub fn get_program_name_safely() -> String
{
	let pathToExeNameOrUnknownString = unsafe { CStr::from_ptr(self::libc::__progname).to_string_lossy().into_owned() };
	match pathToExeNameOrUnknownString.rfind('/')
	{
		None => pathToExeNameOrUnknownString,
		Some(index) => String::from(&pathToExeNameOrUnknownString[(index + 1)..])
	}
}

#[cfg(target_os="windows")]
pub fn get_program_name_safely() -> String
{
	match env::current_exe()
	{
		Err(_) => UnknownProcessName.to_string(),
		Ok(pathBuffer) =>
		{
			match pathBuffer.file_name()
			{
				None => UnspecifiedProcessName.to_string(),
				Some(ref osFileNameString) =>
				{
					let lossy: Cow<str> = osFileNameString.to_string_lossy();
					lossy.to_string()
				}
			}
		}
	};
}

pub fn get_host_name_safely() -> Result<String, Error>
{
	let maximumHostNameLengthExcludingTrailingNul = unsafe { sysconf(_SC_HOST_NAME_MAX) } as usize;
	let mut buffer = Vec::<u8>::with_capacity(maximumHostNameLengthExcludingTrailingNul);

	let pointerToCString = buffer.as_mut_slice().as_mut_ptr() as *mut c_char;

	let result = unsafe { gethostname(pointerToCString, maximumHostNameLengthExcludingTrailingNul as size_t)};
	
	match result
	{
		0 =>
		{
			let stringLengthExcludingTrailingNul = unsafe { strnlen(pointerToCString, maximumHostNameLengthExcludingTrailingNul as size_t) };
			
			unsafe { buffer.set_len(stringLengthExcludingTrailingNul) }
			Ok(String::from_utf8_lossy(buffer.as_slice()).into_owned())
		},
		-1 =>
		{
			Err(Error::last_os_error())
		},
		// ? Is this SOCKET_ERROR on Windows ?
		_ =>
		{
			Err(Error::new(ErrorKind::Other, format!("Returned {}", result)))
		},
	}
}

#[test]
pub fn currentProcessIsValid()
{
	println!("{:?}", *CurrentProcess);
	assert!(false);
}
