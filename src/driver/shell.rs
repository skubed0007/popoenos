use core::clone::Clone;
use core::iter::Iterator;
use core::option::Option::{None, Some};
use core::result::Result::Ok;
use core::str;

use crate::apps::pofetch::pofetch;
use crate::driver::keyboard;
use crate::fs::structure::{BlockDevice, DirEntry, Inode};
use crate::fs::utils::ls;
use crate::polib::print::clear;
use crate::{GLOBAL_DEVICE, PPDev, ROOT_INODE, poprint};

const BUFFER_SIZE: usize = 128;

pub fn shell() {
    {
        let guard = GLOBAL_DEVICE.lock();
        if guard.is_none() {
            return;
        }
        core::mem::drop(guard);
    }

    let mut input_buf = [0u8; BUFFER_SIZE];

    loop {
        poprint!("[bggreen white][[PPOS] ~>[reset] ");
        keyboard::init_keyboard();

        let mut idx = 0;
        loop {
            if let Some(c) = keyboard::read_key() {
                match c {
                    '\x08' if idx > 0 => {
                        idx -= 1;
                        poprint!("\x08 \x08");
                    }
                    '\n' => {
                        poprint!("\n");
                        break;
                    }
                    _ if idx < BUFFER_SIZE => {
                        input_buf[idx] = c as u8;
                        idx += 1;
                        poprint!("{}", c);
                    }
                    _ => {}
                }
            }
        }

        if idx == 0 {
            continue;
        }

        let command = str::from_utf8(&input_buf[..idx]).unwrap_or("").trim();

        if command == "ls" {
            let guard = GLOBAL_DEVICE.lock();
            let dev = guard.as_ref().unwrap();
            ls(dev);
            core::mem::drop(guard);
            continue;
        }

        if command.starts_with("cat ") {
            let fname = command["cat ".len()..].trim();
            let guard = GLOBAL_DEVICE.lock();
            let dev = guard.as_ref().unwrap();

            let mut dir_buf = [0u8; 512];
            dev.read_block(2, &mut dir_buf);

            let ent_sz = core::mem::size_of::<DirEntry>();
            let max = 512 / ent_sz;
            let mut found = None;

            for i in 0..max {
                let base = i * ent_sz;
                let e = unsafe { &*(dir_buf[base..].as_ptr() as *const DirEntry) };
                if e.inode_index != 0 {
                    let len = e.name.iter().position(|&b| b == 0).unwrap_or(32);
                    let name = str::from_utf8(&e.name[..len]).unwrap_or("");
                    if name == fname {
                        found = Some((e.inode_index - 1) as usize);
                        break;
                    }
                }
            }

            if let Some(in_idx) = found {
                let mut ibuf = [0u8; 512];
                dev.read_block(0, &mut ibuf);
                let ptr = &ibuf[in_idx * core::mem::size_of::<Inode>()];
                let inode = unsafe { &*(ptr as *const u8 as *const Inode) };
                let blk = inode.direct_ptrs[0] as u32;

                let mut data = [0u8; 512];
                dev.read_block(blk, &mut data);
                let txt = str::from_utf8(&data).unwrap_or("<non-UTF8>");
                poprint!("{}", txt.trim_start());
            } else {
                poprint!("cat: '{}' not found\n", fname);
            }

            core::mem::drop(guard);
            poprint!("\n");
            continue;
        }

        if command.starts_with("touch ") {
            let fname = command["touch ".len()..].trim();
            if fname.is_empty() {
                poprint!("Usage: touch <name>\n");
                continue;
            }

            let mut guard = GLOBAL_DEVICE.lock();
            let dev = guard.as_mut().unwrap();

            let mut ibuf = [0u8; 512];
            dev.read_block(0, &mut ibuf);

            let isz = core::mem::size_of::<Inode>();
            let slots = 512 / isz;
            let mut slot = None;
            for i in 0..slots {
                let ptr = &ibuf[i * isz] as *const u8 as *const Inode;
                let inode = unsafe { &*ptr };
                if inode.is_used == 0 {
                    slot = Some(i);
                    break;
                }
            }

            if let Some(i) = slot {
                let inode = Inode {
                    mode: 0o100644,
                    size: 0,
                    direct_ptrs: [3,0,0,0,0,0,0,0,0,0,0,0],
                    indirect_ptr: 0,
                    is_used: 1,
                };
                let ib = unsafe {
                    core::slice::from_raw_parts(
                        &inode as *const Inode as *const u8,
                        isz,
                    )
                };
                ibuf[i*isz..(i+1)*isz].copy_from_slice(ib);
                dev.write_block(0, &ibuf);

                let mut ddbuf = [0u8; 512];
                dev.read_block(2, &mut ddbuf);
                let ent_sz = core::mem::size_of::<DirEntry>();
                let max = 512 / ent_sz;
                for j in 0..max {
                    let base = j * ent_sz;
                    let e = unsafe {
                        &*(ddbuf[base..].as_ptr() as *const DirEntry)
                    };
                    if e.inode_index == 0 {
                        let mut ne = DirEntry {
                            name: [0;28],
                            inode_index: (i+1) as u32,
                            _padding: 0,
                        };
                        for (k, &b) in fname.as_bytes().iter().take(31).enumerate() {
                            ne.name[k] = b;
                        }
                        let eb = unsafe {
                            core::slice::from_raw_parts(
                                &ne as *const DirEntry as *const u8,
                                ent_sz,
                            )
                        };
                        ddbuf[base..base+ent_sz].copy_from_slice(eb);
                        dev.write_block(2, &ddbuf);
                        break;
                    }
                }
            }

            core::mem::drop(guard);
            continue;
        }

        if command.starts_with(">> ") {
            let rest = &command[3..];
            let mut sp = rest.splitn(2, ' ');
            let fname = sp.next().unwrap_or("").trim();
            let data = sp.next().unwrap_or("").trim();
            if fname.is_empty() || data.is_empty() {
                poprint!("Usage: >> <file> <text>\n");
                continue;
            }

            let mut guard = GLOBAL_DEVICE.lock();
            let dev = guard.as_mut().unwrap();

            let mut ddbuf = [0u8; 512];
            dev.read_block(2, &mut ddbuf);
            let ent_sz = core::mem::size_of::<DirEntry>();
            let max = 512 / ent_sz;
            let mut inode_index = None;
            for j in 0..max {
                let base = j*ent_sz;
                let e = unsafe {
                    &*(ddbuf[base..].as_ptr() as *const DirEntry)
                };
                if e.inode_index != 0 {
                    let len = e.name.iter().position(|&b|b==0).unwrap_or(32);
                    let nm = str::from_utf8(&e.name[..len]).unwrap_or("");
                    if nm == fname {
                        inode_index = Some((e.inode_index - 1) as usize);
                        break;
                    }
                }
            }

            if let Some(i) = inode_index {
                let mut ibuf = [0u8;512];
                dev.read_block(0, &mut ibuf);
                let ptr = &ibuf[i * core::mem::size_of::<Inode>()];
                let inode = unsafe { &*(ptr as *const u8 as *const Inode) };
                let blk = inode.direct_ptrs[0] as u32;

                let mut db = [0u8;512];
                let bts = data.as_bytes();
                let wl = core::cmp::min(bts.len(), 512);
                db[..wl].copy_from_slice(&bts[..wl]);
                dev.write_block(blk, &db);
            }

            core::mem::drop(guard);
            continue;
        }

        if command == "clear" {
            clear();
            continue;
        }

        if command == "pofetch" || command == "neofetch" {
            pofetch();
            continue;
        }

        if command == "exit" {
            break;
        }

        poprint!("unknown: '{}'\n", command);
    }
}
