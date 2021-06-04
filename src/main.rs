#[cfg(windows)]
extern crate winapi;

use winapi::shared::minwindef::{DWORD};
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS, PROCESSENTRY32W};
use winapi::um::processthreadsapi::{TerminateProcess, OpenProcess};
use std::mem::{zeroed, size_of};
use std::ffi::OsString;
use wio::wide::FromWide;
use regex::Regex;
use lazy_static::lazy_static;
use winapi::um::winnt::PROCESS_ALL_ACCESS;

lazy_static! {
    static ref SPARK_AR_PROCESS_PREFIX: Regex = Regex::new(r"^ARStudio").unwrap();
}

struct Process {
  pid: DWORD,
  name: Option<String>,
}

#[cfg(windows)]
fn get_processes() -> Option<Vec<Process>> {
  let handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };

  if handle == INVALID_HANDLE_VALUE {
    return None
  }

  let mut process_entry: PROCESSENTRY32W = unsafe { zeroed() };
  process_entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;

  match unsafe { Process32FirstW(handle, &mut process_entry) } {
    1 => {
      let mut process_success : i32 = 1;
      let mut processes: Vec<Process> = vec![];

      while process_success == 1 {
        let process_name = OsString::from_wide(&process_entry.szExeFile);

        match process_name.into_string() {
          Ok(s) => {
            processes.push(Process {
              pid: process_entry.th32ProcessID,
              name: Some(s.replace("\u{0}", "")),
            })
          },
          Err(_) => {
            processes.push(Process {
              pid: process_entry.th32ProcessID,
              name: None
            })
          }
        }

        process_success = unsafe { Process32NextW(handle, &mut process_entry) };
      }

      unsafe { CloseHandle(handle) };

      Some(processes)
    },
    0 | _ => {
      unsafe { CloseHandle(handle) };
      None
    }
  }
}

fn main() {
  let processes = match get_processes() {
    Some(p) => p,
    _ => panic!("프로세스 목록을 가져올 수 없습니다.")
  };

  for p in processes {
    let name = match p.name {
      Some(name) => name,
      _ => continue
    };

    if !SPARK_AR_PROCESS_PREFIX.is_match(&name) {
      continue;
    }

    let handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, 0, p.pid) };
    let is_terminated = unsafe { TerminateProcess(handle, 0) };

    if is_terminated != 1 {
      panic!("프로세스를 종료하지 못했습니다.");
    }
  }
}
