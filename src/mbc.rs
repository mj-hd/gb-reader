use crate::board::CubicStyleBoard;
use crate::rom::{MbcType, RomHeader};
use anyhow::Result;
use std::io;
use std::io::{ErrorKind, Read, Seek, SeekFrom};

pub trait MbcReader: Read {
    fn size(&self) -> usize;
    fn status(&self) -> String;
}

pub fn new_mbc_reader<'a>(
    board: &'a mut CubicStyleBoard,
) -> Result<(Box<dyn MbcReader + 'a>, RomHeader)> {
    let header = {
        let mut reader = RomHeaderReader::new(board);

        RomHeader::from_reader(&mut reader)
    }?;

    Ok((
        match header.mbc_type {
            MbcType::RomOnly => Box::new(RomOnlyReader::new(board, header)),
            MbcType::Mbc1 | MbcType::Mbc1Ram | MbcType::Mbc1RamBattery => {
                Box::new(Mbc1Reader::new(board, header))
            }
            MbcType::Mbc2 | MbcType::Mbc2Battery => Box::new(Mbc2Reader::new(board, header)),
            MbcType::Mbc5
            | MbcType::Mbc5Ram
            | MbcType::Mbc5Rumble
            | MbcType::Mbc5RumbleRam
            | MbcType::Mbc5RumbleRamBattery => Box::new(Mbc5Reader::new(board, header)),
            t => {
                unimplemented!("unimplemented mbc: {:?}", t);
            }
        },
        header,
    ))
}

pub struct RomHeaderReader<'a> {
    board: &'a mut CubicStyleBoard,

    addr: u16,
}

impl<'a> RomHeaderReader<'a> {
    pub fn new(board: &'a mut CubicStyleBoard) -> Self {
        Self { board, addr: 0 }
    }

    fn is_valid_addr(&self, addr: i64) -> bool {
        0 <= addr && addr < 0x150
    }
}

impl<'a> Read for RomHeaderReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut n = 0;

        for data in buf.iter_mut() {
            if !self.is_valid_addr(self.addr as i64) {
                break;
            }

            self.board.set_addr(self.addr);

            *data = self
                .board
                .read_byte()
                .map_err(|e| io::Error::new(ErrorKind::BrokenPipe, e))?;

            self.addr += 1;
            n += 1;
        }

        Ok(n)
    }
}

impl<'a> Seek for RomHeaderReader<'a> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let addr = match pos {
            SeekFrom::Start(x) => x as i64,
            SeekFrom::End(x) => self.addr as i64 + x,
            SeekFrom::Current(x) => self.addr as i64 + x,
        };

        if !self.is_valid_addr(addr) {
            return Err(io::Error::new(ErrorKind::AddrNotAvailable, "out of range"));
        }

        self.addr = addr as u16;

        Ok(self.addr as u64)
    }
}

pub struct RomOnlyReader<'a> {
    board: &'a mut CubicStyleBoard,

    addr: u16,
}

impl<'a> MbcReader for RomOnlyReader<'a> {
    fn size(&self) -> usize {
        0x8000
    }

    fn status(&self) -> String {
        format!("{:#04X}", self.addr)
    }
}

impl<'a> RomOnlyReader<'a> {
    pub fn new(board: &'a mut CubicStyleBoard, _header: RomHeader) -> Self {
        Self { board, addr: 0 }
    }

    fn is_valid_addr(&self, addr: i64) -> bool {
        0 <= addr && addr < 0x8000
    }
}

impl<'a> Read for RomOnlyReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut n = 0;

        for data in buf.iter_mut() {
            if !self.is_valid_addr(self.addr as i64) {
                break;
            }

            self.board.set_addr(self.addr);

            *data = self
                .board
                .read_byte()
                .map_err(|e| io::Error::new(ErrorKind::BrokenPipe, e))?;

            self.addr += 1;
            n += 1;
        }

        Ok(n)
    }
}

pub struct Mbc1Reader<'a> {
    board: &'a mut CubicStyleBoard,

    addr: u32,
    size: usize,
    bank: u8,
}

impl<'a> MbcReader for Mbc1Reader<'a> {
    fn size(&self) -> usize {
        self.size
    }

    fn status(&self) -> String {
        format!("BANK#{} {:#04X}", self.bank, self.cur_addr())
    }
}

impl<'a> Mbc1Reader<'a> {
    pub fn new(board: &'a mut CubicStyleBoard, header: RomHeader) -> Self {
        Self {
            board,
            addr: 0,
            size: header.rom_size,
            bank: 0,
        }
    }

    fn cur_addr(&self) -> u16 {
        (if self.addr >= 0x4000 {
            self.addr % 0x4000 + 0x4000
        } else {
            self.addr
        }) as u16
    }

    fn select_rom_bank(&mut self) -> Result<()> {
        let bank_low = self.bank & 0b00011111;
        let bank_high = (self.bank >> 5) & 0b00000011;

        self.board.set_addr(0x2000);
        self.board.write_byte(bank_low)?;

        self.board.set_addr(0x4000);
        self.board.write_byte(bank_high)?;

        Ok(())
    }

    fn is_valid_addr(&self, addr: i64) -> bool {
        0 <= addr && addr < self.size as i64
    }
}

impl<'a> Read for Mbc1Reader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut n = 0;

        for data in buf.iter_mut() {
            if !self.is_valid_addr(self.addr as i64) {
                break;
            }

            if self.addr != 0 && self.addr % 0x4000 == 0 {
                self.bank += 1;

                match self.bank {
                    0x20 | 0x40 | 0x60 => {
                        self.bank += 1;
                    }
                    _ => {}
                }

                self.select_rom_bank()
                    .map_err(|e| io::Error::new(ErrorKind::BrokenPipe, e))?;
            }

            self.board.set_addr(self.cur_addr());
            *data = self
                .board
                .read_byte()
                .map_err(|e| io::Error::new(ErrorKind::BrokenPipe, e))?;

            self.addr += 1;
            n += 1;
        }

        Ok(n)
    }
}

pub struct Mbc2Reader<'a> {
    board: &'a mut CubicStyleBoard,

    addr: u32,
    size: usize,
    bank: u8,
}

impl<'a> MbcReader for Mbc2Reader<'a> {
    fn size(&self) -> usize {
        self.size
    }

    fn status(&self) -> String {
        format!("BANK#{} {:#04X}", self.bank, self.cur_addr())
    }
}

impl<'a> Mbc2Reader<'a> {
    pub fn new(board: &'a mut CubicStyleBoard, header: RomHeader) -> Self {
        Self {
            board,
            addr: 0,
            size: header.rom_size,
            bank: 0,
        }
    }

    fn cur_addr(&self) -> u16 {
        (if self.addr >= 0x4000 {
            self.addr % 0x4000 + 0x4000
        } else {
            self.addr
        }) as u16
    }

    fn select_rom_bank(&mut self) -> Result<()> {
        let bank = self.bank & 0b00001111;

        self.board.set_addr(0x2100);
        self.board.write_byte(bank)?;

        Ok(())
    }

    fn is_valid_addr(&self, addr: i64) -> bool {
        0 <= addr && addr < self.size as i64
    }
}

impl<'a> Read for Mbc2Reader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut n = 0;

        for data in buf.iter_mut() {
            if !self.is_valid_addr(self.addr as i64) {
                break;
            }

            if self.addr != 0 && self.addr % 0x4000 == 0 {
                self.bank += 1;

                self.select_rom_bank()
                    .map_err(|e| io::Error::new(ErrorKind::BrokenPipe, e))?;
            }

            self.board.set_addr(self.cur_addr());
            *data = self
                .board
                .read_byte()
                .map_err(|e| io::Error::new(ErrorKind::BrokenPipe, e))?;

            self.addr += 1;
            n += 1;
        }

        Ok(n)
    }
}

pub struct Mbc3Reader<'a> {
    board: &'a mut CubicStyleBoard,

    addr: u32,
    size: usize,
    bank: u8,
}

impl<'a> MbcReader for Mbc3Reader<'a> {
    fn size(&self) -> usize {
        self.size
    }

    fn status(&self) -> String {
        format!("BANK#{} {:#04X}", self.bank, self.cur_addr())
    }
}

impl<'a> Mbc3Reader<'a> {
    pub fn new(board: &'a mut CubicStyleBoard, header: RomHeader) -> Self {
        Self {
            board,
            addr: 0,
            size: header.rom_size,
            bank: 0,
        }
    }

    fn cur_addr(&self) -> u16 {
        (if self.addr >= 0x4000 {
            self.addr % 0x4000 + 0x4000
        } else {
            self.addr
        }) as u16
    }

    fn select_rom_bank(&mut self) -> Result<()> {
        self.board.set_addr(0x2000);
        self.board.write_byte(self.bank)?;

        Ok(())
    }

    fn is_valid_addr(&self, addr: i64) -> bool {
        0 <= addr && addr < self.size as i64
    }
}

impl<'a> Read for Mbc3Reader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut n = 0;

        for data in buf.iter_mut() {
            if !self.is_valid_addr(self.addr as i64) {
                break;
            }

            if self.addr != 0 && self.addr % 0x4000 == 0 {
                self.bank += 1;

                self.select_rom_bank()
                    .map_err(|e| io::Error::new(ErrorKind::BrokenPipe, e))?;
            }

            self.board.set_addr(self.cur_addr());
            *data = self
                .board
                .read_byte()
                .map_err(|e| io::Error::new(ErrorKind::BrokenPipe, e))?;

            self.addr += 1;
            n += 1;
        }

        Ok(n)
    }
}

pub struct Mbc5Reader<'a> {
    board: &'a mut CubicStyleBoard,

    addr: u32,
    size: usize,
    bank: u16,
}

impl<'a> MbcReader for Mbc5Reader<'a> {
    fn size(&self) -> usize {
        self.size
    }

    fn status(&self) -> String {
        format!("BANK#{} {:#04X}", self.bank, self.cur_addr())
    }
}

impl<'a> Mbc5Reader<'a> {
    pub fn new(board: &'a mut CubicStyleBoard, header: RomHeader) -> Self {
        Self {
            board,
            addr: 0,
            size: header.rom_size,
            bank: 0,
        }
    }

    fn cur_addr(&self) -> u16 {
        (if self.addr >= 0x4000 {
            self.addr % 0x4000 + 0x4000
        } else {
            self.addr
        }) as u16
    }

    fn select_rom_bank(&mut self) -> Result<()> {
        let bank_low = (self.bank & 0xFF) as u8;
        let bank_high = ((self.bank >> 8) & 0b00000001) as u8;

        self.board.set_addr(0x2000);
        self.board.write_byte(bank_low)?;

        self.board.set_addr(0x3000);
        self.board.write_byte(bank_high)?;

        Ok(())
    }

    fn is_valid_addr(&self, addr: i64) -> bool {
        0 <= addr && addr < self.size as i64
    }
}

impl<'a> Read for Mbc5Reader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut n = 0;

        for data in buf.iter_mut() {
            if !self.is_valid_addr(self.addr as i64) {
                break;
            }

            if self.addr != 0 && self.addr % 0x4000 == 0 {
                self.bank += 1;

                self.select_rom_bank()
                    .map_err(|e| io::Error::new(ErrorKind::BrokenPipe, e))?;
            }

            self.board.set_addr(self.cur_addr());
            *data = self
                .board
                .read_byte()
                .map_err(|e| io::Error::new(ErrorKind::BrokenPipe, e))?;

            self.addr += 1;
            n += 1;
        }

        Ok(n)
    }
}
