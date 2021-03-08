#![feature(const_generics)]
#![feature(const_evaluatable_checked)]

use tui::{buffer::Buffer, layout::Rect, widgets::Widget};

struct Matrix<const WIDTH: usize, const HEIGHT: usize> where [bool; WIDTH * HEIGHT] : Sized {
  elems: [bool; WIDTH * HEIGHT],
}

fn fill_block(buf: &mut Buffer, area: Rect, sym: &str) {
  for x in area.left()..area.right() {
    for y in area.top()..area.bottom() {
      // dbg!(x,y);
      buf.get_mut(x, y).set_symbol(sym);
    }
  }
}

impl<const WIDTH: usize, const HEIGHT: usize> Widget for Matrix<WIDTH, HEIGHT>
where [bool; WIDTH * HEIGHT] : Sized
{
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        if area.bottom() - area.top() < HEIGHT as u16 {
          return;
        }

        if area.right() - area.left() < WIDTH as u16 {
          return;
        }

        let scale_factor = ((area.right() - area.left()) / (2 * WIDTH) as u16).min((area.bottom() - area.top()) / HEIGHT as u16);

        for x in 0..WIDTH {
          for y in 0..HEIGHT {
            let offset = x + y * WIDTH;
            let area = Rect::new(x as u16 * 2 * scale_factor, y as u16 * scale_factor, 2 * scale_factor, scale_factor);
            if self.elems[offset] {
              fill_block(buf, area, "█");
            } else {
              fill_block(buf, area, ".");
            }
          }
        }
    }
}


use std::{error::Error, io::{self, Read}};
use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, Terminal};
use std::{time, thread};
use serialport;
use serialport::ClearBuffer;
use termion::event::Key;
use termion::input::TermRead;

use std::sync::{RwLock, Arc};
use std::path::PathBuf;

const ROWS : usize = 11;
const COLS : usize = 10;

use structopt::StructOpt;
#[derive(Debug, StructOpt)]
struct Opts {
  path: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::from_args();

    let mut port = serialport::new(opts.path.to_string_lossy(), 115_200)
      .timeout(time::Duration::from_millis(1000))
    .open().unwrap();

    let stdin = io::stdin();

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let shutdown = Arc::new(RwLock::new(false));
    let shutdown_tx = Arc::clone(&shutdown);
    let input = thread::spawn(move || {
      for k in stdin.keys() {
        let k = k.unwrap();
        match k {
          Key::Ctrl('c') => { *shutdown_tx.try_write().unwrap() = true; return; },
          Key::Char('q') => { *shutdown_tx.try_write().unwrap() = true; return; },
          _ => {},
        };
      }
    });

    let output = thread::spawn(move || {
      loop {
        if *shutdown.read().unwrap() {
          return;
        }

        port.clear(ClearBuffer::Input).unwrap();

        let _ = (&mut port).bytes().take_while(|b| *b.as_ref().unwrap() != '\n' as u8);
        let mut buf = vec![0; ROWS * COLS];
        (&mut port).read_exact(buf.as_mut_slice()).unwrap();

        let mut arr = [false; ROWS * COLS];
        buf.into_iter().zip(arr.iter_mut()).for_each(|(c, a)| *a = c == '1' as u8);

        let m = Matrix::<ROWS, COLS> { elems: arr };
        terminal.draw(|f| f.render_widget(m, f.size())).unwrap();
        thread::sleep(time::Duration::from_millis(20));
      }
    });

    output.join().unwrap();
    Ok(())
}
