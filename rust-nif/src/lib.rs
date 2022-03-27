pub mod nif {
    use std::{
        fs::File,
        io::{BufRead, BufReader, BufWriter, Read, Result, Write},
        path::Path,
    };

    use flate2::{bufread::GzDecoder, write::GzEncoder};
    //Magic number for NIF file
    pub const MAGIC_NUMBER: u32 = 0x4E494600;
    pub const CURRENT_VERSION: u32 = 0x00010000;
    pub const HEADER_SIZE: usize = 0x14;
    pub const FEATURE_FLAGS_COMPRESSION: u32 = 0x1;

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
                Pixel::RGB888(_) => 4,
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
            self.rgb as u8
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
    #[derive(PartialEq, Eq, Ord, PartialOrd, Debug)]
    pub struct Frame {
        pub data: Vec<u8>,
    }
    impl Frame {
        pub fn from(data: Vec<u8>) -> Self {
            Self { data }
        }
        pub fn new(header: Header) -> Self {
            //create a new frame with black pixels
            let data =
                vec![
                    0;
                    header.width as usize * header.height as usize * header.pixel_format.get_size()
                ];
            Self { data }
        }
        pub fn get_pixel(&self, x: u32, y: u32, header: Header) -> Pixel {
            let pixel_size = match header.pixel_format {
                Pixel::RGBA8888(_) => 4,
                Pixel::RGB888(_) => 4,
                Pixel::RGBA4444(_) => 2,
                Pixel::RGB444(_) => 2,
            };
            let pixel_offset = (y * header.width + x) * pixel_size;
            let range = pixel_offset as usize..(pixel_offset + pixel_size) as usize;
            let pixel_data = &self.data[range];

            match &header.pixel_format {
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
        pub fn set_pixel(&mut self, x: u32, y: u32, pixel: Pixel, header: Header) {
            let pixel_size = match header.pixel_format {
                Pixel::RGBA8888(_) => 4,
                Pixel::RGB888(_) => 4,
                Pixel::RGBA4444(_) => 2,
                Pixel::RGB444(_) => 2,
            };
            let pixel_offset = (y * header.width + x) * pixel_size;
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
    // //impl Into PixelIterator for Frame
    // impl<'b> IntoIterator for &'b Frame {
    //     type Item = Pixel;
    //     type IntoIter = PixelIterator<'b>;
    //     fn into_iter(self) -> Self::IntoIter {
    //         PixelIterator {
    //             frame: self,
    //             current_pixel: 0,

    //         }
    //     }
    // }
    pub struct PixelIterator<'b> {
        frame: &'b Frame,
        header: &'b Header,
        current_pixel: u32,
    }
    impl<'b> PixelIterator<'b> {
        fn new(frame: &'b Frame, header: &'b Header) -> Self {
            Self {
                frame,
                current_pixel: 0,
                header,
            }
        }
    }
    //impl iterator for PixelIterator
    impl<'b> Iterator for PixelIterator<'b> {
        type Item = Pixel;
        fn next(&mut self) -> Option<Self::Item> {
            let pixel_size = match self.header.pixel_format {
                Pixel::RGBA8888(_) => 4,
                Pixel::RGB888(_) => 4,
                Pixel::RGBA4444(_) => 2,
                Pixel::RGB444(_) => 2,
            };
            let pixel_offset = self.current_pixel * pixel_size;
            let range = pixel_offset as usize..(pixel_offset + pixel_size) as usize;
            let pixel_data = &self.frame.data[range];
            self.current_pixel += 1;
            if self.current_pixel < self.header.width * self.header.height {
                match self.header.pixel_format {
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
        pub features: u32,
        pub header: Header,
        frames: Vec<Frame>,
    }

    impl Nif {
        pub fn new_default() -> Self {
            Nif {
                version: CURRENT_VERSION,
                features: 0,
                header: Header {
                    width: 0,
                    height: 0,
                    pixel_format: Pixel::RGBA8888(Pixel32U::default()),
                    frame_count: 0,
                    frame_rate: 0.0,
                },
                frames: Vec::new(),
            }
        }
        pub fn new(header: Header) -> Self {
            Nif {
                version: CURRENT_VERSION,
                features: 0,
                header,
                frames: Vec::new(),
            }
        }
        //Returns an iterator over the pixels of the frame at index
        pub fn get_frame(&mut self, index: u32) -> Option<&mut Frame> {
            if index < self.header.frame_count {
                Some(&mut self.frames[index as usize])
            } else {
                None
            }
        }
        pub fn get_frames(&self) -> &Vec<Frame> {
            &self.frames
        }
        pub fn get_frames_mut(&mut self) -> &mut Vec<Frame> {
            &mut self.frames
        }
        pub fn read_from_file(&mut self, path: &Path) -> Result<()> {
            let mut buf = std::io::BufReader::new(std::fs::File::open(path).unwrap());
            let mut magic_number = [0; 4];
            buf.read_exact(&mut magic_number).unwrap();
            let magic_number = u32::from_be_bytes(magic_number);
            if magic_number != MAGIC_NUMBER {
                panic!("Invalid magic number. This is not a NIF file.");
            }

            let mut version_buf = [0; 4];
            buf.read_exact(&mut version_buf).unwrap();
            let version = u32::from_be_bytes(version_buf);
            if version > CURRENT_VERSION {
                panic!("Invalid version. This NIF file is not supported.");
            }
            let mut feature_flags = [0; 4];
            buf.read_exact(&mut feature_flags).unwrap();
            let feature_flags = u32::from_be_bytes(feature_flags);

            self.version = version;
            self.features = feature_flags;
            let mut header_buf = [0; HEADER_SIZE];
            buf.read_exact(&mut header_buf).unwrap();
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
            if feature_flags & FEATURE_FLAGS_COMPRESSION != 0 {
                self.read_compressed(&header, &mut buf)
            } else {
                self.read_uncompressed(&header, &mut buf)
            }
        }

        pub fn read_uncompressed(
            &mut self,
            header: &Header,
            buf: &mut BufReader<File>,
        ) -> Result<()> {
            Ok(())
        }

        pub fn read_compressed(
            &mut self,
            header: &Header,
            buf: &mut BufReader<File>,
        ) -> Result<()> {
            let mut dec = GzDecoder::new(buf);

            let bit_depth = match header.pixel_format {
                Pixel::RGBA8888(_) => 4,
                Pixel::RGB888(_) => 4,
                Pixel::RGBA4444(_) => 2,
                Pixel::RGB444(_) => 2,
            };
            let data_per_frame = header.width as usize * header.height as usize * bit_depth;

            for _ in 0..header.frame_count {
                let mut frame_data = vec![0; data_per_frame];
                dec.read_exact(&mut frame_data).unwrap();
                self.frames.push(Frame { data: frame_data });
            }
            Ok(())
        }

        pub fn write(&self, path: &Path, features: u32) -> std::io::Result<()> {
            let mut buf = std::io::BufWriter::new(std::fs::File::create(path).unwrap());
            buf.write_all(&MAGIC_NUMBER.to_be_bytes()).unwrap();
            //write four empty bytes for feature flags
            //write_version
            buf.write_all(&self.version.to_be_bytes()).unwrap();
            //write features
            buf.write_all(&features.to_be_bytes()).unwrap();

            //write rest of header
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
            header_buf[12..16].copy_from_slice(&self.header.frame_count.to_be_bytes());
            header_buf[16..20].copy_from_slice(&self.header.frame_rate.to_be_bytes());
            buf.write_all(&header_buf).unwrap();

            if features & FEATURE_FLAGS_COMPRESSION != 0 {
                self.write_compressed(&mut buf)
            } else {
                self.write_uncompressed(&mut buf)
            }
        }
        pub fn new_empty_frame(&mut self) {
            self.header.frame_count += 1;
            let hd = self.header;
            let frame = Frame::new(hd);
            self.frames.push(frame);
        }
        pub fn write_compressed(&self, buf: &mut BufWriter<File>) -> Result<()> {
            use flate2::*;
            let mut encoder = GzEncoder::new(buf, Compression::default());
            for frame in &self.frames {
                encoder.write_all(&frame.data)?;
            }
            Ok(())
        }

        pub fn write_uncompressed(&self, buf: &mut BufWriter<File>) -> Result<()> {
            for frame in &self.frames {
                buf.write_all(&frame.data)?;
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod test_super {
    use std::path::Path;

    use rand::Rng;

    use crate::nif::{Header, Nif, Pixel, Pixel32U, FEATURE_FLAGS_COMPRESSION};
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

        let hd = nif.header;
        let frame = nif.get_frame(0).unwrap();
        for i in 0..10 {
            for j in 0..10 {
                frame.set_pixel(
                    i,
                    j,
                    Pixel::RGBA8888(Pixel32U::from_rgba(i as u8, j as u8, 0, 0)),
                    hd,
                );
            }
        }
        for i in 0..10 {
            for j in 0..10 {
                let pixel = frame.get_pixel(i, j, hd);
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
        let hd = nif.header;
        {
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
                        hd,
                    );
                }
            }
        }
        //uncompressed
        nif.write(Path::new("test.nif"), 0).unwrap();
        let mut nif_read = Nif::new_default();
        nif_read.read_from_file(Path::new("test.nif")).unwrap();
        //compare the two nif heads
        {
            assert_eq!(nif.header.width, nif_read.header.width);
            assert_eq!(nif.header.height, nif_read.header.height);
            assert_eq!(nif.header.pixel_format, nif_read.header.pixel_format);
            assert_eq!(nif.header.frame_count, nif_read.header.frame_count);
            assert_eq!(nif.header.frame_rate, nif_read.header.frame_rate);
        }
        //compare the frames
        for frame_pair in nif.get_frames().iter().zip(nif_read.get_frames().iter()) {
            assert_eq!(&frame_pair.0, &frame_pair.1);
        }

        //compressed
        nif.write(Path::new("test_comp.nif"), FEATURE_FLAGS_COMPRESSION)
            .unwrap();

        let mut nif_read_comp = Nif::new_default();
        nif_read_comp
            .read_from_file(Path::new("test_comp.nif"))
            .unwrap();
        //compare the two nif heads
        assert_eq!(nif.header.width, nif_read_comp.header.width);
        assert_eq!(nif.header.height, nif_read_comp.header.height);
        assert_eq!(nif.header.pixel_format, nif_read_comp.header.pixel_format);
        assert_eq!(nif.header.frame_count, nif_read_comp.header.frame_count);
        assert_eq!(nif.header.frame_rate, nif_read_comp.header.frame_rate);
        //compare the frames
        for frame_pair in nif
            .get_frames()
            .iter()
            .zip(nif_read_comp.get_frames().iter())
        {
            assert_eq!(&frame_pair.0, &frame_pair.1);
        }
    }
    #[test]
    fn test_serialize_random() {
        let mut nif = Nif::new(Header {
            width: 400,
            height: 400,
            pixel_format: Pixel::RGBA8888(0.into()),
            frame_count: 0,
            frame_rate: 0.0,
        });
        let mut rng = rand::thread_rng();

        nif.new_empty_frame();
        let hd = nif.header;
        {
            let frame = nif.get_frame(0).unwrap();
            for i in 0..400 {
                for j in 0..400 {
                    frame.set_pixel(
                        i,
                        j,
                        Pixel::RGBA8888(Pixel32U::from_rgba(
                            rng.gen(),
                            rng.gen(),
                            rng.gen(),
                            rng.gen(),
                        )),
                        hd,
                    );
                }
            }
        }
        //uncompressed
        nif.write(Path::new("test_rng.nif"), 0).unwrap();
        let mut nif_read = Nif::new_default();
        nif_read.read_from_file(Path::new("test_rng.nif")).unwrap();
        //compare the two nif heads
        {
            assert_eq!(nif.header.width, nif_read.header.width);
            assert_eq!(nif.header.height, nif_read.header.height);
            assert_eq!(nif.header.pixel_format, nif_read.header.pixel_format);
            assert_eq!(nif.header.frame_count, nif_read.header.frame_count);
            assert_eq!(nif.header.frame_rate, nif_read.header.frame_rate);
        }
        //compare the frames
        for frame_pair in nif.get_frames().iter().zip(nif_read.get_frames().iter()) {
            assert_eq!(&frame_pair.0, &frame_pair.1);
        }

        //compressed
        nif.write(Path::new("test_comp_rng.nif"), FEATURE_FLAGS_COMPRESSION)
            .unwrap();
        let mut nif_read_comp = Nif::new_default();
        nif_read_comp
            .read_from_file(Path::new("test_comp_rng.nif"))
            .unwrap();
        //compare the two nif heads
        assert_eq!(nif.header.width, nif_read_comp.header.width);
        assert_eq!(nif.header.height, nif_read_comp.header.height);
        assert_eq!(nif.header.pixel_format, nif_read_comp.header.pixel_format);
        assert_eq!(nif.header.frame_count, nif_read_comp.header.frame_count);
        assert_eq!(nif.header.frame_rate, nif_read_comp.header.frame_rate);
        //compare the frames
        for frame_pair in nif
            .get_frames()
            .iter()
            .zip(nif_read_comp.get_frames().iter())
        {
            assert_eq!(&frame_pair.0, &frame_pair.1);
        }
    }
}
