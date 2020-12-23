use core::convert::TryInto;	//一种试图消耗自我的转换，它可能昂贵也可能不昂贵。库作者通常不应该直接实现这个特性，而应该更倾向于实现TryFrom trait，它提供了更大的灵活性，并免费提供了等价的尝试实现，这得益于标准库中的全面实现。有关这方面的更多信息，请参阅Into的文档。

use crate::syscall::io::{Io, Mmio, Pio, ReadOnly};
//将位操作和rust的类型系统绑定起来，抽象封装成一个个类型和有意义的名字， 将映设关系固化下来，并且自动完成转化！从而增强语义和表达力，这样会很好用且容易排查错误！
bitflags! {	//Crate [bitflags](https://docs.rs/bitflags/1.2.1/bitflags/) 此Rust Crate可以将一个struct转化为一个bit flags set, 自动完成映设和转化， 此处代码例子出自它的文档， 若要深入了解可去详细阅读之。
    /// Interrupt enable flags
    struct IntEnFlags: u8 {
        const RECEIVED = 1;
        const SENT = 1 << 1;
        const ERRORED = 1 << 2;
        const STATUS_CHANGE = 1 << 3;
        // 4 to 7 are unused
    }
}

bitflags! {
    /// Line status flags
    struct LineStsFlags: u8 {
        const INPUT_FULL = 1;
        // 1 to 4 unknown
        const OUTPUT_EMPTY = 1 << 5;
        // 6 and 7 unknown
    }
}

#[allow(dead_code)]	//抑制 `dead_code` lint
#[repr(packed)]	//在某种程度上打包字段，忽略对齐
pub struct SerialPort<T: Io> {
    /// Data register, read to receive, write to send
    data: T,
    /// Interrupt enable
    int_en: T,
    /// FIFO control
    fifo_ctrl: T,
    /// Line control
    line_ctrl: T,
    /// Modem control
    modem_ctrl: T,
    /// Line status
    line_sts: ReadOnly<T>,
    /// Modem status
    modem_sts: ReadOnly<T>,
}

impl SerialPort<Pio<u8>> {
    pub const fn new(base: u16) -> SerialPort<Pio<u8>> {
        SerialPort {
            data: Pio::new(base),
            int_en: Pio::new(base + 1),
            fifo_ctrl: Pio::new(base + 2),
            line_ctrl: Pio::new(base + 3),
            modem_ctrl: Pio::new(base + 4),
            line_sts: ReadOnly::new(Pio::new(base + 5)),
            modem_sts: ReadOnly::new(Pio::new(base + 6)),
        }
    }
}

impl SerialPort<Mmio<u32>> {
    pub unsafe fn new(base: usize) -> &'static mut SerialPort<Mmio<u32>> {
        &mut *(base as *mut Self)
    }
}

impl<T: Io> SerialPort<T>
where
    T::Value: From<u8> + TryInto<u8>,
{
    pub fn init(&mut self) {
        //TODO: Cleanup
        unsafe {
            self.int_en.write(0x00.into());
            self.line_ctrl.write(0x80.into());
            self.data.write(0x01.into());
            self.int_en.write(0x00.into());
            self.line_ctrl.write(0x03.into());
            self.fifo_ctrl.write(0xC7.into());
            self.modem_ctrl.write(0x0B.into());
            self.int_en.write(0x01.into());
        }
    }

    fn line_sts(&self) -> LineStsFlags {
        LineStsFlags::from_bits_truncate(
            (unsafe { self.line_sts.read() } & 0xFF.into())
                .try_into()	//执行转换
                .unwrap_or(0),	//返回包含的某个值或提供的默认值。传递给unwrap_or的参数被急切地求值;如果传递函数调用的结果，建议使用unwrap_or_else，它是惰性计算的。
        )
    }

    pub fn receive(&mut self) -> Option<u8> {
        if self.line_sts().contains(LineStsFlags::INPUT_FULL) {	//如果结果是包含给定值的Ok值，则返回true。
            Some(
                (unsafe { self.data.read() } & 0xFF.into())
                    .try_into()
                    .unwrap_or(0),
            )
        } else {
            None
        }
    }

    pub fn send(&mut self, data: u8) {
        while !self.line_sts().contains(LineStsFlags::OUTPUT_EMPTY) {}
        unsafe { self.data.write(data.into()) }
    }

    pub fn write(&mut self, buf: &[u8]) {
        for &b in buf {
            match b {
                8 | 0x7F => {
                    self.send(8);
                    self.send(b' ');
                    self.send(8);
                }
                b'\n' => {
                    self.send(b'\r');
                    self.send(b'\n');
                }
                _ => {
                    self.send(b);
                }
            }
        }
    }
}