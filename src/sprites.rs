extern crate glium;
use glium::index::PrimitiveType;
use glium::Surface;

extern crate xml;
use self::xml::reader::EventReader;
use self::xml::reader::events::XmlEvent::*;
use self::xml::name::OwnedName;
use self::xml::attribute::OwnedAttribute;

extern crate image;
extern crate rand;

use std::fmt;
use std::io::{BufReader, Read, Error};
use std::fs::File;
use std::path::Path;
//use std::fs::PathExt;
use std::io::Cursor;
use std::rc::Rc;
use std::ops::Index;

// Loops through the attributes once and pulls out the ones we ask it to. It
// will check that the required ones are there. This could have been done with
// attrs.find but that would be inefficient.
//
// This is probably a really terrible way to do this. It does cut down on lines
// though which is nice.
macro_rules! get_attrs {
    ($attrs:expr, optionals: [$(($oName:pat, $oVar:ident, $oMethod:expr)),*],
     required: [$(($name:pat, $var:ident, $method:expr)),*], $err:expr) => {
        {
            $(let mut $oVar = None;)*
            $(let mut $var = None;)*
            for attr in $attrs.iter() {
                match attr.name.local_name.as_ref() {
                    $($oName => $oVar = $oMethod(attr.value.clone()),)*
                    $($name => $var = $method(attr.value.clone()),)*
                    _ => {}
                }
            }
            if !(true $(&& $var.is_some())*) {
                return Err($err);
            }
            (($($oVar),*), ($($var.unwrap()),*))
        }
    }
}

// Goes through the children of the tag and will call the correct function for
// that child. Closes the tag
//
// Not quite as bad.
macro_rules! parse_tag {
    ($parser:expr, $close_tag:expr, $($open_tag:expr => $open_method:expr),*) => {
        loop {
            match $parser.next() {
                StartElement {name, attributes, ..} => {
                    if false {}
                    $(else if name.local_name == $open_tag {
                        match $open_method(attributes) {
                            Ok(()) => {},
                            Err(e) => return Err(e)
                        };
                    })*
                }
                EndElement {name, ..} => {
                    if name.local_name == $close_tag {
                        break;
                    }
                }
                EndDocument => return Err(SpriteError::PrematureMarkupEnd("Document ended before we expected.".to_string())),
                _ => {}
            }
        }
    }
}


/// Errors which occured when parsing the file
#[derive(Debug)]
pub enum SpriteError {
    /// A attribute was missing, had the wrong type of wasn't formated
    /// correctly.
    MalformedMarkupAttributes(String),
    PrematureMarkupEnd(String),
    FileIo(Error),
    ImageIo(image::ImageError)
}

pub type SpriteResult<T> = Result<T, SpriteError>;

impl fmt::Display for SpriteError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            SpriteError::MalformedMarkupAttributes(ref s) => write!(fmt, "{}", s),
            SpriteError::PrematureMarkupEnd(ref e) => write!(fmt, "{}", e),
            SpriteError::FileIo(ref e) => write!(fmt, "{}", e),
            SpriteError::ImageIo(ref e) => write!(fmt, "{}", e),
        }
    }
}



#[derive(Debug)]
pub struct TextureAtlasEntry {
  pub name: String,
  pub x: usize,
  pub y: usize,
  pub width: usize,
  pub height: usize
}

impl TextureAtlasEntry {
    fn new<R: Read>(parser: &mut EventReader<R>, attrs: Vec<OwnedAttribute>) -> SpriteResult<TextureAtlasEntry>  {
        let ((), (name, x, y, width, height)) = get_attrs!(
          attrs,
          optionals: [],
          //name="hud_0.png" x="230" y="0" width="30" height="38"
          required: [("name", name, |v| Some(v)),
                     ("x", x, |v:String| v.parse().ok()),
                     ("y", y, |v:String| v.parse().ok()),
                     ("width", w, |v:String| v.parse().ok()),
                     ("height", h, |v:String| v.parse().ok())],
          SpriteError::MalformedMarkupAttributes("SubTexture must have a name and selection rect (x, y, width, height) specified.".to_string()));

        Ok(TextureAtlasEntry {
          name: name,
          x: x,
          y: y,
          width: width,
          height: height
        })
    }
}

#[derive(Debug)]
pub struct TextureAtlas {
    pub image_path: String,
    pub texture: Rc<glium::texture::Texture2d>,
    pub entries: Vec<TextureAtlasEntry>
}

impl TextureAtlas {
    fn new<R: Read, P: AsRef<Path>>(display: &glium::Display, parser: &mut EventReader<R>, attrs: Vec<OwnedAttribute>, file_path: P) -> SpriteResult<Rc<TextureAtlas>>  {
        let ((), path) = get_attrs!(
          attrs,
          optionals: [],
          required: [("imagePath", path, |v| Some(v))],
          SpriteError::MalformedMarkupAttributes("TextureAtlas must have an imagePath specified.".to_string()));

        let mut entries = Vec::new();
        parse_tag!(parser, "TextureAtlas",
            "SubTexture" => |attrs| {
                    entries.push(try!(TextureAtlasEntry::new(parser, attrs)));
                    Ok(())
            });

        let mut tex_path = file_path.as_ref().to_path_buf();
        tex_path.pop();
        tex_path.push(&Path::new(&path));

        println!("tex image: {:?}", tex_path);

        let texture = {
            let image =
                image::open(&tex_path).unwrap_or_else(|e| {
                    // TODO: See if we can make this data constant.
                    println!("unable to open {:?}: {:?}", &tex_path, e);
                    let img = image::ImageBuffer::from_fn(32, 32, |x, y| {
                        let cx = (x / 16) % 2;
                        let cy = (y / 16) % 2;
                        if cx != cy {
                            image::Rgba([255u8, 0u8, 0u8, 255u8])
                        } else {
                            image::Rgba([0u8, 0u8, 255u8, 255u8])
                        }
                        //image::Rgba([255u8, 0u8, 0u8, 255u8])
                    });
                    image::DynamicImage::ImageRgba8(img)
                });

            glium::texture::Texture2d::new(display, image)
        };

        Ok(Rc::new(TextureAtlas { image_path: path, texture: Rc::new(texture), entries: entries }))
    }

    pub fn from_file<P: AsRef<Path>>(display: &glium::Display, file_path: P) -> SpriteResult<Rc<TextureAtlas>> {
        let file = try!(File::open(file_path.as_ref()).map_err(SpriteError::FileIo));
        let mut parser = EventReader::new(file);

        loop {
            match parser.next() {
                StartElement { name, attributes, .. } => {
                    if name.local_name == "TextureAtlas" {
                        return TextureAtlas::new(display, &mut parser, attributes, file_path.as_ref());
                    }
                }
                EndDocument => return Err(SpriteError::PrematureMarkupEnd("Document ended before TextureAtlas element was parsed.".to_string())),
                _ => {}
            }
        }
    }

    pub fn get<'a>(&'a self, _index: &str) -> Option<&'a TextureAtlasEntry> {
        self.entries.iter().find(|&e| e.name == _index)
    }
}

struct Sprite {
    position: [f32; 2],
    size: [f32; 2],
    color: [f32; 4],
    tex_rect: [f32; 4],
    texture: Rc<TextureAtlas>
}

pub struct SpriteBatchInst {
    sprites: Vec<Sprite>,
}

pub struct SpriteBatch {
    max_count: usize,
    material: glium::Program,
    vertices: glium::VertexBuffer<SpriteVertex>,
    indicies: glium::IndexBuffer<u16>,
    batches: Vec<SpriteBatchInst>,
}

#[derive(Copy, Clone)]
struct SpriteVertex {
    color: [f32; 4],
    position: [f32; 2],
    uv: [f32; 2],
    tex_id: u32,
}

implement_vertex!(SpriteVertex, position, color, uv, tex_id);

fn ptr_eq<T>(a: *const T, b: *const T) -> bool { a == b }

impl SpriteBatchInst {
    pub fn draw(&mut self, x: f32, y: f32, w: f32, h: f32, u: f32, v: f32, s: f32, t: f32, texture: Rc<TextureAtlas>) {
        // TODO: Add depth sorting, different modes based on details passed into begin.
        // TODO: Don't let sprites vec grow past max_count
        //let color: (f32, f32, f32) = (rand::random(), rand::random(), rand::random());
        // let dimensions = self.frame.get_dimensions();
        // let vw = dimensions.0 as f32;
        // let vh = dimensions.1 as f32;
        self.sprites.push(Sprite {
            position: [x, y],
            size: [w, h],
            color: [1.0, 1.0, 1.0, 1.0],
            tex_rect: [u, v, s, t],
            //tex_rect: [0.0, 0.0, 32.0, 32.0],
            texture: texture
        });
    }

    pub fn draw_entry(&mut self, x: f32, y: f32, w: f32, h: f32, texture: Rc<TextureAtlas>, entry: &TextureAtlasEntry) {
        // TODO: Add depth sorting, different modes based on details passed into begin.
        // TODO: Don't let sprites vec grow past max_count
        //let color: (f32, f32, f32) = (rand::random(), rand::random(), rand::random());
        // let dimensions = self.frame.get_dimensions();
        // let vw = dimensions.0 as f32;
        // let vh = dimensions.1 as f32;
        self.draw(x, y, w, h, entry.x as f32, entry.y as f32, entry.width as f32, entry.height as f32, texture);
    }


}

impl SpriteBatch {
    pub fn new(display: &glium::Display, max_batch_size: usize) -> SpriteResult<SpriteBatch> {
        let program = program!(display,
            140 => {
                vertex: include_str!("../assets/shaders/sprite_batch.140.vsh"),
                fragment: include_str!("../assets/shaders/sprite_batch.140.fsh"),
            },
            110 => {
                vertex: include_str!("../assets/shaders/sprite_batch.110.vsh"),
                fragment: include_str!("../assets/shaders/sprite_batch.110.fsh"),
            },
            100 => {
                vertex: include_str!("../assets/shaders/sprite_batch.100.vsh"),
                fragment: include_str!("../assets/shaders/sprite_batch.100.fsh"),
            },
        ).unwrap();

        let mut vb = glium::VertexBuffer::empty_dynamic(display, max_batch_size * 4);
        let mut ib_data = Vec::with_capacity(max_batch_size * 6);

        for (num, _) in vb.map().chunks_mut(4).enumerate() {
            let num = num as u16;
            ib_data.push(num * 4 + 0);
            ib_data.push(num * 4 + 1);
            ib_data.push(num * 4 + 2);
            ib_data.push(num * 4 + 1);
            ib_data.push(num * 4 + 3);
            ib_data.push(num * 4 + 2);
        }

        let ib = glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, ib_data);
        Ok (SpriteBatch {
            max_count: max_batch_size,
            material: program,
            vertices: vb,
            indicies: ib,
            batches: Vec::new()
        })
    }

    fn display_to_viewport(x: f32, y: f32, vw: f32, vh: f32) -> (f32, f32) {
        let vx = x / vw - 1.0;
        let vy = y / vh - 1.0;
        (vx, vy)
    }

    pub fn begin<F>(&mut self, scope: F) where F: FnOnce(&mut SpriteBatchInst) {
        self.batches.clear(); // TODO: Move to separate call.
        let mut batch =
            SpriteBatchInst {
                sprites: Vec::with_capacity(self.max_count)
            };
        scope(&mut batch);
        self.batches.push(batch);
    }

    pub fn end<'f>(&mut self, frame: &'f mut glium::Frame) {
        // TODO: WIP
        // TODO: Check bounds of sprite/sprite_verts.
        // TODO: Draw x batches to makes sure all sprites are drawn.
        let (vw, vh) = frame.get_dimensions();
        let vw = vw as f32;
        let vh = vh as f32;
        let hvw = vw / 2.0;
        let hvh = vh / 2.0;
        let mut offset = 0;

        // TODO: Sort sprites by active texture.
        let mut texture = None;

        {
            let verts = &mut self.vertices;
            let mut vert_mapped = verts.map();
            let vert_chunks = &mut vert_mapped.chunks_mut(4);

            for batch in self.batches.iter() {
                let sprite_count = batch.sprites.len();
                let verts = vert_chunks.skip(offset).take(sprite_count);
                offset += sprite_count;

                for (i, sprite_verts) in verts.enumerate() {
                    let sprite = &batch.sprites[i];
                    texture = Some(sprite.texture.clone());
                    let x = (sprite.position[0] * 2.0 - vw) / vw;
                    let y = ((sprite.position[1] * 2.0 - vh) / vh) * -1.0;
                    let w = sprite.size[0] / vw;
                    let h = sprite.size[1] / vh;
                    let color = sprite.color;

                    let tex = &sprite.texture.texture;
                    let tw = tex.get_width() as f32;
                    let th = tex.get_height().unwrap() as f32;
                    let tl = sprite.tex_rect[0] / tw;
                    let tt = (th - sprite.tex_rect[1]) / th;
                    let tr = (sprite.tex_rect[0] + sprite.tex_rect[2]) / tw;
                    let tb = (th - sprite.tex_rect[1] - sprite.tex_rect[3]) / th;

                    sprite_verts[0].position[0] = x - w;
                    sprite_verts[0].position[1] = y + h;
                    sprite_verts[0].uv[0] = tl;
                    sprite_verts[0].uv[1] = tt;
                    sprite_verts[0].color = color;

                    sprite_verts[1].position[0] = x + w;
                    sprite_verts[1].position[1] = y + h;
                    sprite_verts[1].uv[0] = tr;
                    sprite_verts[1].uv[1] = tt;
                    sprite_verts[1].color = color;

                    sprite_verts[2].position[0] = x - w;
                    sprite_verts[2].position[1] = y - h;
                    sprite_verts[2].uv[0] = tl;
                    sprite_verts[2].uv[1] = tb;
                    sprite_verts[2].color = color;

                    sprite_verts[3].position[0] = x + w;
                    sprite_verts[3].position[1] = y - h;
                    sprite_verts[3].uv[0] = tr;
                    sprite_verts[3].uv[1] = tb;
                    sprite_verts[3].color = color;

                    // for i in 0..4 {
                    //     println!("sprite_verts[{}].uv: {},{}", i, sprite_verts[i].uv[0], sprite_verts[i].uv[1]);
                    // }
                }
            }
        }


        let ib_slice = self.indicies.slice(0 .. offset * 6).unwrap();
        let tex = texture.unwrap();
        let uniforms = uniform! {
            tex: tex.texture.sampled()//.magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
        };

        frame.draw(&self.vertices, &ib_slice,
                    &self.material, &uniforms, &Default::default()).unwrap();

    }
}
