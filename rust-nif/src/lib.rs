pub mod nif {
    use std::{
        io::{Read, Write},
        path::Path,
    };
    //Magic number for NIF file
    const MAGIC_NUMBER: u32 = 0x4E494600;
    const CURRENT_VERSION: u32 = 0x00010000;
    const HEADER_SIZE: usize = 0x14;
    //describes how the pixel data is stored
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Pixel {
        RGBA8888(Pixel32U),
        RGB888(Pixel32U),
        RGBA4444(Pixel16U),
        RGB444(Pixel16U),
    }
    impl Pixel {
        fn get_size(&self) -> usize {
            match self {
                Pixel::RGBA8888(_) => 4,
                Pixel::RGB888(_) => 3,
                Pixel::RGBA4444(_) => 2,
                Pixel::RGB444(_) => 2,
            }
        }
    }
    #[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Pixel32U {
        rgba: u32,
    }
    impl Pixel32U {
        fn from_u32(rgba: u32) -> Pixel32U {
            Pixel32U { rgba }
        }

        pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
            Pixel32U {
                rgba: (a as u32) << 24 | (b as u32) << 16 | (g as u32) << 8 | (r as u32),
            }
        }
        pub fn r(&self) -> u8 {
            (self.rgba >> 24) as u8
        }
        pub fn g(&self) -> u8 {
            (self.rgba >> 16) as u8
        }
        pub fn b(&self) -> u8 {
            (self.rgba >> 8) as u8
        }
        pub fn a(&self) -> u8 {
            self.rgba as u8
        }
        pub fn set_r(&mut self, r: u8) {
            self.rgba = (self.rgba & 0x00FFFFFF) | ((r as u32) << 24);
        }
        pub fn set_g(&mut self, g: u8) {
            self.rgba = (self.rgba & 0xFF00FFFF) | ((g as u32) << 16);
        }
        pub fn set_b(&mut self, b: u8) {
            self.rgba = (self.rgba & 0xFFFF00FF) | ((b as u32) << 8);
        }
        pub fn set_a(&mut self, a: u8) {
            self.rgba = (self.rgba & 0xFFFFFF00) | (a as u32);
        }
        pub fn get(&self) -> u32 {
            self.rgba
        }
    }
    #[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Pixel16U {
        rgb: u16,
    }
    impl Pixel16U {
        fn from_u16(rgb: u16) -> Pixel16U {
            Pixel16U { rgb }
        }
        fn new(r: u8, g: u8, b: u8) -> Pixel16U {
            Pixel16U {
                rgb: (b as u16) << 12 | (g as u16) << 8 | (r as u16),
            }
        }
        fn r(&self) -> u8 {
            (self.rgb >> 12) as u8
        }
        fn g(&self) -> u8 {
            (self.rgb >> 8) as u8
        }
        fn b(&self) -> u8 {
            (self.rgb >> 0) as u8
        }
        fn set_r(&mut self, r: u8) {
            self.rgb = (self.rgb & 0x0FFF) | ((r as u16) << 12);
        }
        fn set_g(&mut self, g: u8) {
            self.rgb = (self.rgb & 0xF0FF) | ((g as u16) << 8);
        }
        fn set_b(&mut self, b: u8) {
            self.rgb = (self.rgb & 0xFFF0) | ((b as u16) << 0);
        }
        fn get(&self) -> u16 {
            self.rgb
        }
    }
    //impl into u16 for Pixel16I
    impl Into<u16> for Pixel16U {
        fn into(self) -> u16 {
            self.rgb
        }
    }

    //impl Into<u32> for Pixel32I
    impl Into<u32> for Pixel32U {
        fn into(self) -> u32 {
            self.rgba
        }
    }
    //impl from u32 for Pixel32I
    impl From<u32> for Pixel32U {
        fn from(rgba: u32) -> Pixel32U {
            Pixel32U { rgba }
        }
    }
    //impl from u16 for Pixel16I
    impl From<u16> for Pixel16U {
        fn from(rgb: u16) -> Pixel16U {
            Pixel16U { rgb }
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct Header {
        pub width: u32,
        pub height: u32,
        pub pixel_format: Pixel,
        pub frame_count: u32,
        pub frame_rate: f32,
    }
    pub struct Frame {
        data: Vec<u8>,
        header: Box<Header>,
    }
    impl Frame {
        pub fn from(data: Vec<u8>, header: &Header) -> Self {
            let head_box = Box::new(*header);
            Self {
                data,
                header: head_box,
            }
        }
        pub fn new(header: &Header) -> Self {
            //create a new frame with black pixels
            let data =
                vec![
                    0;
                    header.width as usize * header.height as usize * header.pixel_format.get_size()
                ];
            Self {
                data,
                header: Box::new(*header),
            }
        }
        pub fn get_pixel(&self, x: u32, y: u32) -> Pixel {
            let pixel_size = match &self.header.pixel_format {
                Pixel::RGBA8888(_) => 4,
                Pixel::RGB888(_) => 4,
                Pixel::RGBA4444(_) => 2,
                Pixel::RGB444(_) => 2,
            };
            let pixel_offset = (y * self.header.width + x) * pixel_size;
            let range = pixel_offset as usize..(pixel_offset + pixel_size) as usize;
            let pixel_data = &self.data[range];

            match &self.header.pixel_format {
                Pixel::RGBA8888(_) => Pixel::RGBA8888(Pixel32U::from_u32(u32::from_be_bytes(
                    pixel_data.try_into().unwrap(),
                ))),
                Pixel::RGB888(_) => Pixel::RGB888(Pixel32U::from_u32(u32::from_be_bytes(
                    pixel_data.try_into().unwrap(),
                ))),
                Pixel::RGBA4444(_) => Pixel::RGBA4444(Pixel16U::from_u16(u16::from_be_bytes(
                    pixel_data.try_into().unwrap(),
                ))),
                Pixel::RGB444(_) => Pixel::RGB444(Pixel16U::from_u16(u16::from_be_bytes(
                    pixel_data.try_into().unwrap(),
                ))),
            }
        }
        pub fn set_pixel(&mut self, x: u32, y: u32, pixel: Pixel) {
            let pixel_size = match self.header.pixel_format {
                Pixel::RGBA8888(_) => 4,
                Pixel::RGB888(_) => 4,
                Pixel::RGBA4444(_) => 2,
                Pixel::RGB444(_) => 2,
            };
            let pixel_offset = (y * self.header.width + x) * pixel_size;
            let range = pixel_offset as usize..(pixel_offset + pixel_size) as usize;
            let pixel_data = &mut self.data[range];

            match pixel {
                Pixel::RGBA8888(val) => {
                    pixel_data.copy_from_slice(&val.get().to_le_bytes());
                }
                Pixel::RGB888(val) => pixel_data.copy_from_slice(&val.get().to_le_bytes()),
                Pixel::RGBA4444(val) => {
                    pixel_data.copy_from_slice(&val.get().to_le_bytes());
                }
                Pixel::RGB444(val) => pixel_data.copy_from_slice(&val.get().to_le_bytes()),
            }
        }
    }
    //impl Into PixelIterator for Frame
    impl<'b> IntoIterator for &'b Frame {
        type Item = Pixel;
        type IntoIter = PixelIterator<'b>;
        fn into_iter(self) -> Self::IntoIter {
            PixelIterator {
                frame: self,
                current_pixel: 0,
            }
        }
    }
    pub struct PixelIterator<'b> {
        frame: &'b Frame,
        current_pixel: u32,
    }
    impl<'b> PixelIterator<'b> {
        fn new(frame: &'b Frame) -> Self {
            Self {
                frame,
                current_pixel: 0,
            }
        }
    }
    //impl iterator for PixelIterator
    impl<'b> Iterator for PixelIterator<'b> {
        type Item = Pixel;
        fn next(&mut self) -> Option<Self::Item> {
            let pixel_size = match self.frame.header.pixel_format {
                Pixel::RGBA8888(_) => 4,
                Pixel::RGB888(_) => 4,
                Pixel::RGBA4444(_) => 2,
                Pixel::RGB444(_) => 2,
            };
            let pixel_offset = self.current_pixel * pixel_size;
            let range = pixel_offset as usize..(pixel_offset + pixel_size) as usize;
            let pixel_data = &self.frame.data[range];
            self.current_pixel += 1;
            if self.current_pixel < self.frame.header.width * self.frame.header.height {
                match self.frame.header.pixel_format {
                    Pixel::RGBA8888(_) => Some(Pixel::RGBA8888(
                        u32::from_be_bytes(pixel_data.try_into().unwrap()).into(),
                    )),
                    Pixel::RGB888(_) => Some(Pixel::RGB888(
                        u32::from_be_bytes(pixel_data.try_into().unwrap()).into(),
                    )),
                    Pixel::RGBA4444(_) => Some(Pixel::RGBA4444(
                        u16::from_be_bytes(pixel_data.try_into().unwrap()).into(),
                    )),
                    Pixel::RGB444(_) => Some(Pixel::RGB444(
                        u16::from_be_bytes(pixel_data.try_into().unwrap()).into(),
                    )),
                }
            } else {
                None
            }
        }
    }

    pub struct Nif {
        pub version: u32,
        pub header: Header,
        data: Vec<Frame>,
    }

    impl Nif {
        pub fn new_default() -> Self {
            Nif {
                version: CURRENT_VERSION,
                header: Header {
                    width: 0,
                    height: 0,
                    pixel_format: Pixel::RGBA8888(Pixel32U::default()),
                    frame_count: 0,
                    frame_rate: 0.0,
                },
                data: Vec::new(),
            }
        }
        pub fn new(header: Header) -> Self {
            Nif {
                version: CURRENT_VERSION,
                header,
                data: Vec::new(),
            }
        }
        //Returns an iterator over the pixels of the frame at index
        pub fn get_frame(&mut self, index: u32) -> Option<&mut Frame> {
            if index < self.header.frame_count {
                Some(&mut self.data[index as usize])
            } else {
                None
            }
        }
        pub fn read_from_file(&mut self, path: &Path) {
            let mut buffered_reader = std::io::BufReader::new(std::fs::File::open(path).unwrap());
            let mut magic_number = [0; 4];
            buffered_reader.read_exact(&mut magic_number).unwrap();
            let magic_number = u32::from_be_bytes(magic_number);
            if magic_number != MAGIC_NUMBER {
                panic!("Invalid magic number. This is not a NIF file.");
            }
            let mut version_buf = [0; 4];
            buffered_reader.read_exact(&mut version_buf).unwrap();
            let version = u32::from_be_bytes(version_buf);
            if version > CURRENT_VERSION {
                panic!("Invalid version. This NIF file is not supported.");
            }
            self.version = version;
            let mut header_buf = [0; HEADER_SIZE];
            buffered_reader.read_exact(&mut header_buf).unwrap();
            let header: Header = Header {
                width: u32::from_be_bytes(header_buf[0..4].try_into().unwrap()),
                height: u32::from_be_bytes(header_buf[4..8].try_into().unwrap()),
                pixel_format: match u32::from_be_bytes(header_buf[8..12].try_into().unwrap()) {
                    0 => Pixel::RGBA8888(0.into()),
                    1 => Pixel::RGB888(0.into()),
                    2 => Pixel::RGBA4444(0.into()),
                    3 => Pixel::RGB444(0.into()),
                    _ => panic!("Invalid pixel format."),
                },
                frame_count: u32::from_be_bytes(header_buf[12..16].try_into().unwrap()),
                frame_rate: f32::from_be_bytes(header_buf[16..20].try_into().unwrap()),
            };
            self.header = header;
            let mut frames = Vec::with_capacity(self.header.frame_count as usize);
            for _ in 0..self.header.frame_count {
                let data = match self.header.pixel_format {
                    Pixel::RGBA8888(_) => {
                        let mut data =
                            vec![0; self.header.width as usize * self.header.height as usize * 4];
                        buffered_reader.read_exact(&mut data).unwrap();
                        data
                    }
                    Pixel::RGB888(_) => {
                        let mut data =
                            vec![0; self.header.width as usize * self.header.height as usize * 3];
                        buffered_reader.read_exact(&mut data).unwrap();
                        data
                    }
                    Pixel::RGBA4444(_) => {
                        let mut data =
                            vec![0; self.header.width as usize * self.header.height as usize * 2];
                        buffered_reader.read_exact(&mut data).unwrap();
                        data
                    }
                    Pixel::RGB444(_) => {
                        let mut data =
                            vec![0; self.header.width as usize * self.header.height as usize * 2];
                        buffered_reader.read_exact(&mut data).unwrap();
                        data
                    }
                };

                frames.push(Frame {
                    data,
                    header: Box::from(self.header),
                });
            }
        }

        pub fn write_to_file(&self, path: &Path) {
            let mut buffered_writer = std::io::BufWriter::new(std::fs::File::create(path).unwrap());
            let mut magic_number = [0; 4];
            magic_number.copy_from_slice(&self.version.to_be_bytes());
            buffered_writer.write_all(&magic_number).unwrap();
            let mut version_buf = [0; 4];
            version_buf.copy_from_slice(&self.version.to_be_bytes());
            //write four empty bytes for feature flags
            buffered_writer.write_all(&[0, 0, 0, 0]).unwrap();
            buffered_writer.write_all(&version_buf).unwrap();
            let mut header_buf = [0; HEADER_SIZE];
            header_buf[0..4].copy_from_slice(&self.header.width.to_be_bytes());
            header_buf[4..8].copy_from_slice(&self.header.height.to_be_bytes());
            match self.header.pixel_format {
                Pixel::RGBA8888(_) => {
                    header_buf[8..12].copy_from_slice(&0_u32.to_be_bytes());
                }
                Pixel::RGB888(_) => {
                    header_buf[8..12].copy_from_slice(&1_u32.to_be_bytes());
                }
                Pixel::RGBA4444(_) => {
                    header_buf[8..12].copy_from_slice(&2_u32.to_be_bytes());
                }
                Pixel::RGB444(_) => {
                    header_buf[8..12].copy_from_slice(&3_u32.to_be_bytes());
                }
            }

            buffered_writer.write_all(&header_buf).unwrap();
            for frame in &self.data {
                buffered_writer.write_all(&frame.data).unwrap();
            }
        }
        pub fn new_empty_frame(&mut self) {
            self.header.frame_count += 1;
            let hd: &Header = &self.header;
            let frame = Frame::new(hd);
            self.data.push(frame);
        }
    }
}

#[cfg(test)]
mod test_super {
    use std::path::Path;

    use crate::nif::{Header, Nif, Pixel, Pixel32U};
    #[test]
    fn test_access_pixels() {
        let mut nif = Nif::new(Header {
            width: 10,
            height: 10,
            pixel_format: Pixel::RGBA8888(0.into()),
            frame_count: 0,
            frame_rate: 0.0,
        });

        nif.new_empty_frame();

        let frame = nif.get_frame(0).unwrap();
        for i in 0..10 {
            for j in 0..10 {
                frame.set_pixel(
                    i,
                    j,
                    Pixel::RGBA8888(Pixel32U::from_rgba(i as u8, j as u8, 0, 0)),
                );
            }
        }
        for i in 0..10 {
            for j in 0..10 {
                let pixel = frame.get_pixel(i, j);
                match pixel {
                    Pixel::RGBA8888(p) => {
                        assert_eq!(p.r(), i as u8);
                        assert_eq!(p.g(), j as u8);
                        assert_eq!(p.b(), 0);
                        assert_eq!(p.a(), 0);
                    }
                    _ => panic!("Invalid pixel type."),
                }
            }
        }
    }
    #[test]
    fn test_serialize() {
        let mut nif = Nif::new(Header {
            width: 400,
            height: 400,
            pixel_format: Pixel::RGBA8888(0.into()),
            frame_count: 0,
            frame_rate: 0.0,
        });

        nif.new_empty_frame();

        let frame = nif.get_frame(0).unwrap();
        for i in 0..400 {
            for j in 0..400 {
                frame.set_pixel(
                    i,
                    j,
                    Pixel::RGBA8888(Pixel32U::from_rgba(
                        (i % 0xFF) as u8,
                        (j & 0xFF) as u8,
                        0,
                        0,
                    )),
                );
            }
        }
        nif.write_to_file(Path::new("test.nif"));

        let mut nif_read = Nif::new_default();
        nif_read.read_from_file(Path::new("test.nif"));
        //compare the two nif heads
        assert_eq!(nif.header.width, nif_read.header.width);
        assert_eq!(nif.header.height, nif_read.header.height);
        assert_eq!(nif.header.pixel_format, nif_read.header.pixel_format);
        assert_eq!(nif.header.frame_count, nif_read.header.frame_count);
        assert_eq!(nif.header.frame_rate, nif_read.header.frame_rate);
    }
}
