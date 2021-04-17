use anyhow::Result;
use rppal::gpio::{Gpio, OutputPin};
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use std::thread::sleep;
use std::time::Duration;

enum Pin {
    Rd = 20,
    Wr = 3,
    Cs = 26,
    Rst = 23,

    Addr0 = 16,
    Addr1 = 19,
    Addr2 = 13,
    Addr3 = 12,
    Addr4 = 6,
    Addr5 = 5,
    Addr6 = 25,
    Addr7 = 24,
    Addr8 = 22,
    Addr9 = 27,
    Addr10 = 18,
    Addr11 = 17,
    Addr12 = 15,
    Addr13 = 14,
    Addr14 = 4,
    Addr15 = 21,
}

const DEV_ID: u8 = 0;

const MCP23X08_IODIR: u8 = 0x00;
const MCP23X08_IOCON: u8 = 0x05;
const MCP23X08_GPIO: u8 = 0x09;

const CS_WAIT: u64 = 3;
const RD_WAIT: u64 = 4;
const WR_WAIT_BEFORE: u64 = 1;
const WR_WAIT_AFTER: u64 = 5;

const CMD_WRITE: u8 = 0x40;
const CMD_READ: u8 = 0x41;

const IOCON_INIT: u8 = 0x20;

#[derive(PartialEq)]
enum DataDir {
    Input,
    Output,
}

pub struct CubicStyleBoard {
    gpio: Gpio,
    spi: Spi,

    rd: OutputPin,
    wr: OutputPin,
    cs: OutputPin,
    rst: OutputPin,

    addr: [OutputPin; 16],
    data_dir: DataDir,
}

impl CubicStyleBoard {
    pub fn new() -> Result<Self> {
        let gpio = Gpio::new()?;

        let rd = (&gpio).get(Pin::Rd as u8)?.into_output();
        let wr = (&gpio).get(Pin::Wr as u8)?.into_output();
        let cs = (&gpio).get(Pin::Cs as u8)?.into_output();
        let rst = (&gpio).get(Pin::Rst as u8)?.into_output();
        let addr = [
            (&gpio).get(Pin::Addr0 as u8)?.into_output(),
            (&gpio).get(Pin::Addr1 as u8)?.into_output(),
            (&gpio).get(Pin::Addr2 as u8)?.into_output(),
            (&gpio).get(Pin::Addr3 as u8)?.into_output(),
            (&gpio).get(Pin::Addr4 as u8)?.into_output(),
            (&gpio).get(Pin::Addr5 as u8)?.into_output(),
            (&gpio).get(Pin::Addr6 as u8)?.into_output(),
            (&gpio).get(Pin::Addr7 as u8)?.into_output(),
            (&gpio).get(Pin::Addr8 as u8)?.into_output(),
            (&gpio).get(Pin::Addr9 as u8)?.into_output(),
            (&gpio).get(Pin::Addr10 as u8)?.into_output(),
            (&gpio).get(Pin::Addr11 as u8)?.into_output(),
            (&gpio).get(Pin::Addr12 as u8)?.into_output(),
            (&gpio).get(Pin::Addr13 as u8)?.into_output(),
            (&gpio).get(Pin::Addr14 as u8)?.into_output(),
            (&gpio).get(Pin::Addr15 as u8)?.into_output(),
        ];

        Ok(Self {
            gpio,
            spi: Spi::new(Bus::Spi0, SlaveSelect::Ss1, 4000000, Mode::Mode0)?,
            rd,
            wr,
            cs,
            rst,
            addr,
            data_dir: DataDir::Input,
        })
    }

    pub fn init(&mut self) -> Result<()> {
        self.rd.set_high();
        self.wr.set_high();
        self.rst.set_high();
        self.cs.set_high();

        // SEQOPの禁止
        self.write_mcp_byte(MCP23X08_IOCON, IOCON_INIT)?;

        // DataをINPUTへ設定
        self.mcp_into_input()?;

        Ok(())
    }

    pub fn set_addr(&mut self, addr: u16) {
        for i in 0..16 {
            let pin = &mut self.addr[i];
            if addr & (1 << i) > 0 {
                pin.set_high();
            } else {
                pin.set_low();
            }
        }
    }

    pub fn read_byte(&mut self) -> Result<u8> {
        self.mcp_into_input()?;

        self.set_write(false);
        self.set_read(true);
        self.set_cs(true);

        let data = self.read_mcp_byte(MCP23X08_GPIO)?;

        self.set_read(false);
        self.set_cs(false);

        Ok(data)
    }

    pub fn write_byte(&mut self, val: u8) -> Result<()> {
        self.mcp_into_output()?;

        self.set_read(false);
        self.set_cs(true);

        self.write_mcp_byte(MCP23X08_GPIO, val)?;

        self.set_write(true);
        self.set_write(false);
        self.set_cs(false);

        Ok(())
    }

    fn set_write(&mut self, val: bool) {
        if self.wr.is_set_low() == val {
            return;
        }

        sleep(Duration::from_micros(WR_WAIT_BEFORE));

        if val {
            self.wr.set_low();
        } else {
            self.wr.set_high();
        }

        sleep(Duration::from_micros(WR_WAIT_AFTER));
    }

    fn set_read(&mut self, val: bool) {
        if self.rd.is_set_low() == val {
            return;
        }

        if val {
            self.rd.set_low();
        } else {
            self.rd.set_high();
        }

        sleep(Duration::from_micros(RD_WAIT));
    }

    fn set_cs(&mut self, val: bool) {
        if self.cs.is_set_low() == val {
            return;
        }

        if val {
            self.cs.set_low();
        } else {
            self.cs.set_high();
        }

        sleep(Duration::from_micros(CS_WAIT));
    }

    fn mcp_into_output(&mut self) -> Result<()> {
        if self.data_dir != DataDir::Output {
            self.write_mcp_byte(MCP23X08_IODIR, 0x00)?;
            self.data_dir = DataDir::Output;
        }

        Ok(())
    }

    fn mcp_into_input(&mut self) -> Result<()> {
        if self.data_dir != DataDir::Input {
            self.write_mcp_byte(MCP23X08_IODIR, 0xFF)?;
            self.data_dir = DataDir::Input;
        }

        Ok(())
    }

    fn write_mcp_byte(&mut self, reg: u8, val: u8) -> Result<()> {
        let mut data: [u8; 3] = [0; 3];

        data[0] = CMD_WRITE | ((DEV_ID & 7) << 1);
        data[1] = reg;
        data[2] = val;

        self.spi.write(&data)?;

        Ok(())
    }

    fn read_mcp_byte(&mut self, reg: u8) -> Result<u8> {
        let mut data: [u8; 3] = [0; 3];

        data[0] = CMD_READ | ((DEV_ID & 7) << 1);
        data[1] = reg;

        let mut buffer: [u8; 3] = [0; 3];

        self.spi.transfer(&mut buffer, &data)?;

        Ok(buffer[2])
    }
}

impl Drop for CubicStyleBoard {
    fn drop(&mut self) {
        self.rd.set_high();
        self.wr.set_high();
        self.cs.set_high();
        self.rst.set_high();
    }
}
