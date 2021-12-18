use std::{path::Path, io::Read};

//Magic number for NIF file
const MAGIC_NUMBER: u32 = 0x4E494600;
const CURRENT_VERSION: u32 = 0x00010000;
const HEADER_SIZE: usize = 0x28;
//describes how the pixel data is stored
enum Pixel {
    RGBA8888(u32),
    RGB888(u32),
    RGBA4444(u16),
    RGB444(u16),
}
struct Nif {
    version: u32,
    header: Header,
    data: Vec<u8>,
}
struct Header {
    width: u32,
    height: u32,
    pixel_format: Pixel,
    frame_count: u32,
    frame_rate: f32,
}
struct Frame {
    data: Vec<Pixel>,
}
impl Nif{
    fn new_default() -> Self {
        Nif {
            version: CURRENT_VERSION,
            header: Header {
                width: 0,
                height: 0,
                pixel_format: Pixel::RGBA8888(0),
                frame_count: 0,
                frame_rate: 0.0,
            },
            data: Vec::new(),
        }

        
    }
    fn new(header:Header, data:Vec<u8>) -> Self {
        Nif {
            version: CURRENT_VERSION,
            header: header,
            data: data,
        }
    }
    fn read_from_file(path: &Path) -> Self {
      let mut buffered_reader = std::io::BufReader::new(std::fs::File::open(path).unwrap());
        let mut magic_number = [0; 4];
        buffered_reader.read_exact(&mut magic_number).unwrap();
        let magic_number = u32::from_be_bytes(magic_number);
        if magic_number != MAGIC_NUMBER {
            panic!("Invalid magic number. This is not a NIF file.");
        }
        let version_buf= [0; 4];
        buffered_reader.read_exact(&mut version_buf).unwrap();
        let version = u32::from_be_bytes(version_buf);
        if version > CURRENT_VERSION {
            panic!("Invalid version. This NIF file is not supported.");
        }
        let header_buf = [0; HEADER_SIZE];
        buffered_reader.read_exact(&mut header_buf).unwrap();
        let header=unsafe{
            let header_ptr = header_buf.as_ptr() as *const Header;
            &*header_ptr
        };
        let frames_buf=[0;header.frame_count*header.width*header.height*];
        }


  
    }
}
