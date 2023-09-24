use crate::{try_gread_vec_with, try_gwrite_vec_with};
use scroll::{ctx, Endian, Pread, Pwrite, BE, LE};

#[derive(Debug)]
pub struct KV6Format {
    pub magic: u32, // big endian
    pub x_size: u32,
    pub y_size: u32,
    pub z_size: u32,

    pub x_pivot: f32,
    pub y_pivot: f32,
    pub z_pivot: f32,
    pub voxels: Vec<VoxelData>, // length = num_voxels
    pub xlen: Vec<u32>,         // cached data for speed in Build engine, length = x_size
    pub ylen: Vec<Vec<u16>>, // more cached data for speed in Build engine, length[1] = x_size, length[2] = y_size
}

#[derive(Debug, Default)]
pub struct VoxelData {
    pub red: u8,   // 0..255
    pub green: u8, // 0..255
    pub blue: u8,  // 0..255
    pub dummy: u8, // always 128, was probably once an alpha value

    pub height: u16,     // little endian,
    pub visibility: u8,  // low 6 bits are hidden surface removal info
    pub normalindex: u8, // should probably ignore
}

impl Default for KV6Format {
    fn default() -> Self {
        Self { magic: 0x4b76786c, x_size: Default::default(), y_size: Default::default(), z_size: Default::default(), x_pivot: Default::default(), y_pivot: Default::default(), z_pivot: Default::default(), voxels: Default::default(), xlen: Default::default(), ylen: Default::default() }
    }
}

impl ctx::TryIntoCtx<Endian> for KV6Format {
    type Error = scroll::Error;

    fn try_into_ctx(self, bytes: &mut [u8], ctx: Endian) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        bytes.gwrite_with(self.magic, offset, BE)?;
        bytes.gwrite_with(self.x_size, offset, ctx)?;
        bytes.gwrite_with(self.y_size, offset, ctx)?;
        bytes.gwrite_with(self.z_size, offset, ctx)?;

        bytes.gwrite_with(self.x_pivot, offset, ctx)?;
        bytes.gwrite_with(self.y_pivot, offset, ctx)?;
        bytes.gwrite_with(self.z_pivot, offset, ctx)?;

        bytes.gwrite_with(self.voxels.len() as u32, offset, ctx)?;
        try_gwrite_vec_with!(bytes, offset, self.voxels, ctx);

        try_gwrite_vec_with!(bytes, offset, self.xlen, ctx);
        // TODO: Writing for Vec<Vec<u16>>
        // try_gwrite_vec_with!(bytes, offset, self.ylen, ctx);

        Ok(*offset)
    }
}

impl ctx::TryIntoCtx<Endian> for VoxelData {
    type Error = scroll::Error;

    fn try_into_ctx(self, bytes: &mut [u8], ctx: Endian) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        bytes.gwrite_with(self.red, offset, ctx)?;
        bytes.gwrite_with(self.green, offset, ctx)?;
        bytes.gwrite_with(self.blue, offset, ctx)?;
        bytes.gwrite_with(self.dummy, offset, ctx)?;
        bytes.gwrite_with(self.height, offset, LE)?;
        bytes.gwrite_with(self.visibility, offset, ctx)?;
        bytes.gwrite_with(self.normalindex, offset, ctx)?;

        Ok(*offset)
    }
}

impl<'a> ctx::TryFromCtx<'a, Endian> for KV6Format {
    type Error = scroll::Error;
    fn try_from_ctx(src: &'a [u8], endian: Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;

        let magic: u32 = src.gread_with(offset, BE)?;
        let x_size: u32 = src.gread_with(offset, endian)?;
        let y_size: u32 = src.gread_with(offset, endian)?;
        let z_size: u32 = src.gread_with(offset, endian)?;

        let x_pivot: f32 = src.gread_with(offset, endian)?;
        let y_pivot: f32 = src.gread_with(offset, endian)?;
        let z_pivot: f32 = src.gread_with(offset, endian)?;

        let num_voxels: u32 = src.gread_with(offset, endian)?;
        // let data: Vec<u8> = try_gread_vec_with!(src, offset, size, endian);
        let voxels: Vec<VoxelData> = try_gread_vec_with!(src, offset, num_voxels, endian);
        let xlen: Vec<u32> = try_gread_vec_with!(src, offset, x_size, endian);

        Ok((
            KV6Format {
                magic,
                x_size,
                y_size,
                z_size,
                x_pivot,
                y_pivot,
                z_pivot,
                voxels,
                xlen,
                ylen: Default::default(),
            },
            *offset,
        ))
    }
}

impl<'a> ctx::TryFromCtx<'a, Endian> for VoxelData {
    type Error = scroll::Error;
    fn try_from_ctx(src: &'a [u8], endian: Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;

        let red: u8 = src.gread_with(offset, BE)?;
        let green: u8 = src.gread_with(offset, endian)?;
        let blue: u8 = src.gread_with(offset, endian)?;
        let dummy: u8 = src.gread_with(offset, endian)?;

        let height: u16 = src.gread_with(offset, LE)?;
        let visibility: u8 = src.gread_with(offset, endian)?;
        let normalindex: u8 = src.gread_with(offset, endian)?;

        Ok((
            VoxelData {
                red,
                green,
                blue,
                dummy,
                height,
                visibility,
                normalindex,
            },
            *offset,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::KV6Format;
    use scroll::Pread;
    use std::{
        fs::File,
        io::{BufReader, Read},
    };

    #[test]
    fn test_read_header() {
        let f = File::open("data/grenade.kv6").unwrap();
        let mut reader = BufReader::new(f);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap();

        let data = buffer.pread::<KV6Format>(0).unwrap();

        assert_eq!(data.magic, 0x4b76786c);
    }

    #[test]
    fn test_read_sizes() {
        let f = File::open("data/grenade.kv6").unwrap();
        let mut reader = BufReader::new(f);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap();

        let data = buffer.pread::<KV6Format>(0).unwrap();

        assert_eq!(data.x_size, 6);
        assert_eq!(data.y_size, 6);
        assert_eq!(data.z_size, 9);
    }

    #[test]
    fn test_read_pivots() {
        let f = File::open("data/grenade.kv6").unwrap();
        let mut reader = BufReader::new(f);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap();

        let data = buffer.pread::<KV6Format>(0).unwrap();

        assert_eq!(data.x_pivot, 2.5);
        assert_eq!(data.y_pivot, 2.5);
        assert_eq!(data.z_pivot, 3.5);
    }

    #[test]
    fn test_read_voxels() {
        let f = File::open("data/grenade.kv6").unwrap();
        let mut reader = BufReader::new(f);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap();

        let data = buffer.pread::<KV6Format>(0).unwrap();
        assert_eq!(data.voxels.len(), 74);
    }

    #[test]
    fn test_read_xlen() {
        let f = File::open("data/grenade.kv6").unwrap();
        let mut reader = BufReader::new(f);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap();

        let data = buffer.pread::<KV6Format>(0).unwrap();
        assert_eq!(data.xlen.len() as u32, data.x_size);
    }
}
