use std::io::{self, Read, Write};
use std::str;
use byteorder::{ByteOrder, BigEndian};
use ::frame::Frame;
use ::tag::{self, Version};
use ::stream::encoding::Encoding;
use ::stream::frame;
use ::stream::unsynch;

pub fn decode<R>(reader: &mut R, unsynchronisation: bool) -> ::Result<Option<(usize, Frame)>>
    where R: io::Read {
    let mut frame_header = [0; 6];
    let nread = reader.read(&mut frame_header)?;
    if nread < frame_header.len() || frame_header[0] == 0x00 {
        return Ok(None);
    }
    let id = str::from_utf8(&frame_header[0..3])?;

    let sizebytes = &frame_header[3..6];
    let read_size = ((sizebytes[0] as u32) << 16) | ((sizebytes[1] as u32) << 8) | sizebytes[2] as u32;
    let content = super::decode_content(reader.take(read_size as u64), id, false, unsynchronisation)?;
    let frame = Frame::with_content(id, content);
    Ok(Some((6 + read_size as usize, frame)))
}

pub fn encode(writer: &mut Write, frame: &Frame, unsynchronisation: bool) -> ::Result<usize> {
    let mut content_buf = Vec::new();
    frame::content::encode(&mut content_buf, frame.content(), tag::Id3v22, Encoding::UTF16)?;
    assert_ne!(0, content_buf.len());
    let id = frame.id_for_version(Version::Id3v22)
        .ok_or(::Error::new(::ErrorKind::InvalidInput, "Unable to downgrade frame ID to ID3v2.2"))?;
    assert_eq!(3, id.len());
    writer.write_all(id.as_bytes())?;
    let mut size_buf = [0; 4];
    BigEndian::write_u32(&mut size_buf, content_buf.len() as u32);
    writer.write_all(&size_buf[1..4])?;
    if unsynchronisation {
        unsynch::encode_vec(&mut content_buf);
    }
    writer.write_all(&content_buf)?;
    Ok(7 + content_buf.len())
}
