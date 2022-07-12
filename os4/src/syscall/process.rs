//! Process management syscalls

use crate::config::{MAX_SYSCALL_NUM, PAGE_SIZE};
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next, TaskStatus, get_cur_task_info, current_user_token, get_tcb_ref_mut};
use crate::timer::get_time_us;
use crate::mm::{translated_refmut, VirtAddr, MapPermission};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

#[derive(Clone, Copy)]
pub struct TaskInfo {
    pub status: TaskStatus,
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    pub time: usize,
}

pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

// YOUR JOB: 引入虚地址后重写 sys_get_time
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    let us = get_time_us();
    let t = translated_refmut(current_user_token(), _ts);

    t.sec = us / 1000000;
    t.usec = us % 10000000;
    
    0
}

// CLUE: 从 ch4 开始不再对调度算法进行测试~
pub fn sys_set_priority(_prio: isize) -> isize {
    -1
}

// YOUR JOB: 扩展内核以实现 sys_mmap 和 sys_munmap
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {

    if _start & (PAGE_SIZE-1) != 0 {
        // 没有按照页对齐
        return -1;
    }

    if _port & (!0x07) != 0 || (_port & 0x07) == 0 {
        return -1;
    } 

    let start_va: VirtAddr = VirtAddr::from(_start).floor().into();
    let end_va: VirtAddr = VirtAddr::from(_start + _len - 1).ceil().into();


    let mut permission = MapPermission::empty();
    permission.set(MapPermission::U, true);

    if _port & 0x01 != 0 {
        permission.set(MapPermission::R, true);
    }

    if _port & 0x02 != 0 {
        permission.set(MapPermission::W, true);
    }

    if _port & 0x04 != 0 {
        permission.set(MapPermission::X, true);
    }


    if !get_tcb_ref_mut(|tcb| {
        tcb.memory_set.mmap(start_va.into(), end_va.into(), permission)
    })  {
        return -1;
    }
    
    0
}

pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    if !get_tcb_ref_mut(|tcb| {
        tcb.memory_set.munmap(_start.into(), (_start + _len).into())
    })  {
        return -1;
    }
    0
}

// YOUR JOB: 引入虚地址后重写 sys_task_info
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    let (status, stats) = get_cur_task_info();
    let t = translated_refmut(current_user_token(), ti);
    *t = TaskInfo{
        status,
        syscall_times: stats.syscall_times.clone(),
        time: (get_time_us() - stats.first_run_time) / 1000,
    };
    0
}
