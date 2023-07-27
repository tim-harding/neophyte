use async_trait::async_trait;
use nvim_rs::{compat::tokio::Compat, Handler, Neovim, UiAttachOptions, Value};
use png::Encoder;
use rusttype::{point, Font, Scale};
use std::{
    fs::{self, File},
    io::BufWriter,
    path::Path,
    process::Stdio,
};
use tokio::process::{ChildStdin, Command};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

#[derive(Clone)]
struct NeovimHandler {}

#[async_trait]
impl Handler for NeovimHandler {
    type Writer = Compat<ChildStdin>;

    async fn handle_request(
        &self,
        name: String,
        _args: Vec<Value>,
        _neovim: Neovim<Self::Writer>,
    ) -> Result<Value, Value> {
        println!("Request: {name}");
        Ok(Value::Nil)
    }

    async fn handle_notify(&self, name: String, args: Vec<Value>, _neovim: Neovim<Self::Writer>) {
        println!("Notify: {name}");
        for arg in args {
            match Event::try_from(arg) {
                Ok(event) => println!("{event:?}"),
                Err(e) => match e {
                    EventParseError::UnknownEvent(name) => eprintln!("Unknown event: {name}"),
                    _ => eprintln!("{e}"),
                },
            }
        }
        println!();
    }
}

#[tokio::main]
async fn main() {
    let path = Path::new(r"/home/tim/temp.png");
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = Encoder::new(w, 512, 256);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut w = encoder.write_header().unwrap();

    let mut data = [255u8; 256 * 512 * 3];
    for x in 0..512 {
        for y in 0..256 {
            let p = (y * 512 + x) * 3;
            data[p] = x as u8;
            data[p + 1] = y as u8;
        }
    }

    let font_path = Path::new(r"/usr/share/fonts/TTF/CaskaydiaCoveNerdFont-Regular.ttf");
    let font_data = fs::read(&font_path).unwrap();
    let font = Font::try_from_vec(font_data).unwrap();
    let height = 48f32;

    let scale = Scale::uniform(height);

    // The ascent is the highest point of any glyph. We shift down so that first line doesn't clip.
    let v_metrics = font.v_metrics(scale);
    let offset = point(0.0, v_metrics.ascent);

    let glyphs: Vec<_> = font.layout("rusttype", scale, offset).collect();
    for glyph in glyphs {
        if let Some(bb) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, v| {
                let x = (x as i32 + bb.min.x) as usize;
                let y = (y as i32 + bb.min.y) as usize;
                if x > 512 || y > 256 {
                    return;
                }
                let v = 1.0 - v;
                let p = (y as usize * 512 + x as usize) * 3;
                data[p] = ((data[p] as f32) * v) as u8;
                data[p + 1] = ((data[p + 1] as f32) * v) as u8;
                data[p + 2] = ((data[p + 2] as f32) * v) as u8;
            });
        }
    }

    let mut child = Command::new("nvim")
        .arg("--embed")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let handler = NeovimHandler {};
    let reader = child.stdout.take().unwrap();
    let writer = child.stdin.take().unwrap();
    let (neovim, io) = Neovim::new(reader.compat(), writer.compat_write(), handler);
    let io_handle = tokio::spawn(io);

    let mut options = UiAttachOptions::new();
    options.set_linegrid_external(true);
    neovim.ui_attach(512, 512, &options).await.unwrap();

    tokio::spawn(async move {
        neovim.input("iThings<esc>").await.unwrap();
    });

    match io_handle.await {
        Err(join_error) => eprintln!("Error joining IO loop: '{}'", join_error),
        Ok(Err(error)) => {
            if !error.is_channel_closed() {
                eprintln!("Error: '{}'", error);
            }
        }
        Ok(Ok(())) => {}
    };

    w.write_image_data(&data).unwrap();
}

#[derive(Debug, Clone, Copy)]
enum Event {
    GridResize(GridResize),
}

impl TryFrom<Value> for Event {
    type Error = EventParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Array(array) => {
                let mut iter = array.into_iter();
                let event_name = iter.next().ok_or(EventParseError::Malformed)?;
                let args = iter.next().ok_or(EventParseError::Malformed)?;
                if !iter.next().is_none() {
                    return Err(EventParseError::Malformed);
                }
                match event_name {
                    Value::String(s) => match s.as_str() {
                        Some(s) => match s {
                            "grid_resize" => Ok(Self::GridResize(GridResize::try_from(args)?)),
                            _ => Err(EventParseError::UnknownEvent(s.to_string())),
                        },
                        None => Err(EventParseError::Malformed),
                    },
                    _ => Err(EventParseError::Malformed),
                }
            }
            _ => Err(EventParseError::Malformed),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
enum EventParseError {
    #[error("Event is malformed")]
    Malformed,
    #[error("Received an unrecognized event name: {0}")]
    UnknownEvent(String),
    #[error("{0}")]
    GridResize(#[from] GridResizeParseError),
}

#[derive(Debug, Clone, Copy)]
struct GridResize {
    pub grid: u64,
    pub width: u64,
    pub height: u64,
}

impl TryFrom<Value> for GridResize {
    type Error = GridResizeParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Array(array) => {
                let mut iter = array.into_iter().map(pos_int).flatten();
                let grid = iter.next().ok_or(GridResizeParseError)?;
                let width = iter.next().ok_or(GridResizeParseError)?;
                let height = iter.next().ok_or(GridResizeParseError)?;
                Ok(Self {
                    grid,
                    width,
                    height,
                })
            }
            _ => Err(GridResizeParseError),
        }
    }
}

fn pos_int(value: Value) -> Option<u64> {
    match value {
        Value::Integer(n) => n.as_u64(),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Failed to parse grid_resize event")]
struct GridResizeParseError;
