#![feature(const_generics)]
#![feature(const_evaluatable_checked)]

use tui::widgets::Widget;

struct Matrix<const WIDTH: usize, const HEIGHT: usize> where [bool; WIDTH * HEIGHT] : Sized {
  elems: [bool; WIDTH * HEIGHT],
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


        for x in 0..WIDTH {
          for y in 0..HEIGHT {
            let offset = x + y * WIDTH;

            if self.elems[offset] {
              buf.get_mut(area.left() + x as u16, area.top() + y as u16).set_symbol("█");
            } else {
              buf.get_mut(area.left() + x as u16, area.top() + y as u16).set_symbol(".");
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

fn main() -> Result<(), Box<dyn Error>> {
    let mut port = serialport::new("/dev/cu.usbmodem14401", 115_200)
      .timeout(time::Duration::from_millis(250))
    .open().unwrap();

    let mut stdin = io::stdin();

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
        let mut buf = vec![0; 10 * 11];
        (&mut port).read_exact(buf.as_mut_slice()).unwrap();

        let mut arr = [false; 10 * 11];
        buf.into_iter().zip(arr.iter_mut()).for_each(|(c, a)| *a = c == '1' as u8);

        let m = Matrix::<11, 10> { elems: arr };
        terminal.draw(|f| f.render_widget(m, f.size())).unwrap();
        thread::sleep(time::Duration::from_millis(90));
      }
    });

    input.join().unwrap();
    output.join().unwrap();
    Ok(())
}
