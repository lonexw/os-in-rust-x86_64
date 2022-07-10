use lazy_static::lazy_static;
use volatile::Volatile;
use spin::Mutex;
use core::fmt;

#[allow(dead_code)]		// 禁用未使用變量警告
#[repr(u8)]				// 注记标注的枚举类型，都会以一个 u8 的形式存储
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
	Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

// ColorCode: 顏色代碼字節
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
	fn new(foreground: Color, background: Color) -> ColorCode {
		ColorCode((background as u8) << 4 | (foreground as u8))
	}
}

// ScreenChar: 屏幕字符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// #[repr(C)] 标记结构体；
// 这将按 C 语言约定的顺序布局它的成员变量
// 让我们能正确地映射内存片段
#[repr(C)]
struct ScreenChar {
	ascii_character: u8,
	color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
	chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
	column_position: usize,
	color_code: ColorCode,
	buffer: &'static mut Buffer,	// 借用应该在整个程序的运行期间有效
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
        	// 接收到换行符 \n 的时候，将所有的字符向上位移一行
            b'\n' => self.new_line(),
            byte => {
            	// 寫滿換行
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
    	for row in 1..BUFFER_HEIGHT {
    		// 遍历每个屏幕上的字符，把每个字符移动到它上方一行的相应位置
    		// 省略了对第 0 行的枚举过程
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    // 向对应的缓冲区写入空格字符，清空一整行的字符位置
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    // 打印整个字符串，把它转换为字节并依次输出
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // 可以是能打印的 ASCII 码字节，也可以是换行符
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // 不包含在上述范围之内的字节
                _ => self.write_byte(0xfe),
            }

        }
    }
}

// 实现 core::fmt::Write trait，格式化宏 write! 和 writeln!
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        // 属于 Result 枚举类型中的 Ok，包含一个值为 () 的变量
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[cfg(test)]
use crate::{serial_print, serial_println};

#[test_case]
fn test_println_simple() {
    serial_print!("test_println... ");
    println!("test_println_simple output");
    serial_println!("[ok]");
}

#[test_case]
fn test_println_many() {
    serial_print!("test_println_many... ");
    for _ in 0..200 {
        println!("test_println_many output");
    }
    serial_println!("[ok]");
}

#[test_case]
fn test_println_output() {
    serial_print!("test_println_output... ");

    let s = "Some test string that fits on a single line";
    println!("{}", s);
    for (i, c) in s.chars().enumerate() {
        let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char::from(screen_char.ascii_character), c);
    }

    serial_println!("[ok]");
}
