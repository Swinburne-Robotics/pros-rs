#![no_std]
#![feature(alloc_error_handler)]
#![feature(negative_impls)]
#![feature(const_option)]

//! # PROS bindings
//! This library contains safe Rust bindings for the Vex PROS environment. It should be used in conjunction with this template crate [github.com/serxka/pros-rs-template](https://github.com/serxka/pros-rs-template).
//!
//! This library is currently a work in progress, expect breaking API changes or
//! undefined behaviour, or just straight up missing features.
//!
//! # Hello World
//! ```
//! #![no_std]
//! #![no_main]
//!
//! #[macro_use]
//! extern crate pros;
//! use pros::prelude::*;
//!
//! struct VexRobot;
//!
//! impl Robot for VexRobot {
//! 	fn new(devices: Devices) -> Self {
//! 		println!("Hello World!");
//! 		VexRobot
//! 	}
//! }
//! robot!(VexRobot);
//! ```

extern crate alloc;
#[macro_use]
extern crate smallvec;

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub mod bindings {
	//! 1:1 bindings of the equivalent C functions in PROS.
	//!
	//! You are not meant to use this, however they are available in-case there
	//! is functionality lacking from the bindings currently. If there is
	//! something you wish to be added, please open an issue on the Github
	//! issues page.

	include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[doc(hidden)]
#[macro_use]
pub mod macros;
mod util;

pub mod devices;
pub mod ports;
pub mod prelude {
	//! Common types and macros that can all be conveniently imported at once.

	pub use crate::devices::{controller::*, motor::*, DeviceError, Devices};
	pub use crate::ports::*;
	pub use crate::rtos::{
		tasks::{CompetitionState, CompetitionTask, Task},
		time::{Instant, Interval},
		Mutex,
	};
	pub use crate::Robot;
	pub use core::time::Duration;
	pub use libc_print::std_name::*;
}
pub mod rtos;

// Re-export math crate
pub mod math {
	//! Re-export of the internal math crate for user use.

	pub use math::*;
}

/// This trait is used so that `pros-rs` knows which functions it should call
/// for the tasks that are addressed out by the competition manager.
///
/// This means that every single one of these functions will be called from a
/// separate task and that they can return early at any possible time. The
/// exception to this is [`Robot::new()`] which will block all other functions
/// from being called until it returns.
pub trait Robot {
	/// The entry point to the users program, a structure containing owned
	/// values to every possible device is passed into this function. The ports
	/// that will be needed throughout the whole of the programs execution
	/// should be moved and called with the relevant `into_<>` function to
	/// convert them into specific devices.
	///
	/// [`Robot::new()`] should ideally return as soon as possible to allow
	/// operator control and autonomous code to run as soon as possible. This
	/// means keeping the initialisation code lean and not waiting around
	/// collecting sensor data. Collection of sensor data should be performed
	/// from another task.
	fn new(devices: devices::Devices) -> Self;

	#[allow(unused_variables)]
	fn competition_init(&'static self, state: rtos::tasks::CompetitionState) {}

	#[allow(unused_variables)]
	fn disabled(&'static self, state: rtos::tasks::CompetitionState) {}

	#[allow(unused_variables)]
	fn autonomous(&'static self, state: rtos::tasks::CompetitionState) {}

	#[allow(unused_variables)]
	fn opcontrol(&'static self, state: rtos::tasks::CompetitionState) {}
}

// LANGUAGE ITEMS
use core::alloc::{GlobalAlloc, Layout};

// Wrap the newlib's allocator to Rust's global allocator
struct LibcAlloc;
unsafe impl GlobalAlloc for LibcAlloc {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		libc::memalign(layout.align(), layout.size()) as *mut u8
	}
	unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
		libc::free(ptr as *mut core::ffi::c_void)
	}
}

#[global_allocator]
static ALLOC: LibcAlloc = LibcAlloc;

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
	panic!("alloc failed: {:?}", layout);
}

// TODO: Printing to screen for easy debugging
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
	libc_print::libc_eprintln!("panic has occured: {:?}", info);

	unsafe {
		// Go through and stop motors regardless if they are actually motors or not
		for i in 1..21 {
			bindings::motor_set_brake_mode(i, devices::motor::BrakeMode::Coast.into());
			bindings::motor_move(i, 0);
		}

		libc::exit(1);
	}
}
