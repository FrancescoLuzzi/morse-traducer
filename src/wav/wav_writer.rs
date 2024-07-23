use crate::polyphonia::SAMPLE_RATE;
use std::{
    borrow::Borrow,
    cell::RefMut,
    io::{self, Seek, SeekFrom, Write},
    marker::PhantomData,
};

const HEADER_SIZE: usize = 44;
const RIFF_SIZE: u64 = 4;
// header - riff - 4bytes of file length
const OFFSET_SIZE: u32 = HEADER_SIZE as u32 - RIFF_SIZE as u32 - 4_u32;

pub trait WavOutBuffer: Write + Seek {}
impl<T: Write + Seek> WavOutBuffer for T {}

pub trait WavWriterStatus {}
pub struct Created {}
pub struct Initiated {}
impl WavWriterStatus for Created {}
impl WavWriterStatus for Initiated {}

#[derive(Debug, Clone, Copy)]
pub struct WavOptions {
    written_data: u32,
    chunk_size: u32,
    pcm: u16,
    num_channels: u16,
    sample_rate: u32,
    bytes_per_second: u32,
    bytes_per_sample: u16,
    bits_per_sample: u16,
}

impl Default for WavOptions {
    fn default() -> Self {
        Self {
            written_data: 0,
            chunk_size: 16,
            pcm: 1,
            num_channels: 1,
            sample_rate: SAMPLE_RATE,
            bytes_per_second: SAMPLE_RATE * 2,
            bytes_per_sample: 2,
            bits_per_sample: 16,
        }
    }
}

impl WavOptions {
    pub fn align(&mut self) {
        self.bytes_per_sample = self.bits_per_sample / 8;
        self.bytes_per_second = self.sample_rate * self.bytes_per_sample as u32;
    }
}

impl From<&WavOptions> for [u8; HEADER_SIZE] {
    fn from(val: &WavOptions) -> Self {
        let mut out = [0_u8; HEADER_SIZE];
        out[..4].copy_from_slice(b"RIFF");
        // data length + header_size - riff - 4bytes
        out[4..8].copy_from_slice(&(val.written_data + OFFSET_SIZE).to_le_bytes());
        out[8..12].copy_from_slice(b"WAVE");
        out[12..16].copy_from_slice(b"fmt ");
        out[16..20].copy_from_slice(&val.chunk_size.to_le_bytes());
        out[20..22].copy_from_slice(&val.pcm.to_le_bytes());
        out[22..24].copy_from_slice(&val.num_channels.to_le_bytes());
        out[24..28].copy_from_slice(&val.sample_rate.to_le_bytes());
        out[28..32].copy_from_slice(&val.bytes_per_second.to_le_bytes());
        out[32..34].copy_from_slice(&val.bytes_per_sample.to_le_bytes());
        out[34..36].copy_from_slice(&val.bits_per_sample.to_le_bytes());
        out[36..40].copy_from_slice(b"data");
        // data length
        out[40..44].copy_from_slice(&(val.written_data + 32).to_le_bytes());
        out
    }
}

#[derive(Default)]
pub struct WavBuilder {
    wav_opts: WavOptions,
}

impl WavBuilder {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn sample_rate(&mut self, sample_rate: u32) -> &mut Self {
        self.wav_opts.sample_rate = sample_rate;
        self
    }
    pub fn chunk_size(&mut self, chunk_size: u32) -> &mut Self {
        self.wav_opts.chunk_size = chunk_size;
        self
    }
    pub fn pcm(&mut self, pcm: u16) -> &mut Self {
        self.wav_opts.pcm = pcm;
        self
    }
    pub fn num_channels(&mut self, num_channels: u16) -> &mut Self {
        self.wav_opts.num_channels = num_channels;
        self
    }
    pub fn bits_per_sample(&mut self, bits_per_sample: u16) -> &mut Self {
        self.wav_opts.bits_per_sample = bits_per_sample;
        self
    }
    pub fn set_output<'a, T: WavOutBuffer>(
        &mut self,
        out_buffer: &'a mut T,
    ) -> WavWriter<'a, T, Created> {
        self.wav_opts.align();
        WavWriter::new(out_buffer, self.wav_opts)
    }
}

pub struct WavWriter<'a, T, S>
where
    T: WavOutBuffer,
    S: WavWriterStatus,
{
    out_buffer: &'a mut T,
    wav_opts: WavOptions,
    header_position: u64,
    status: PhantomData<S>,
}

impl<'a, T, Src> WavWriter<'a, T, Src>
where
    T: WavOutBuffer,
    Src: WavWriterStatus,
{
    fn transition<Dest: WavWriterStatus>(self) -> WavWriter<'a, T, Dest> {
        let WavWriter {
            out_buffer,
            wav_opts,
            header_position: start_pos,
            status: _,
        } = self;

        WavWriter {
            out_buffer,
            wav_opts,
            header_position: start_pos,
            status: PhantomData,
        }
    }
}

impl<'a, T> WavWriter<'a, T, Created>
where
    T: WavOutBuffer,
{
    fn new(out_buffer: &'a mut T, wav_opts: WavOptions) -> Self {
        Self {
            out_buffer,
            wav_opts,
            header_position: 0,
            status: PhantomData,
        }
    }
}

impl<'a, T> WavWriter<'a, T, Created>
where
    T: WavOutBuffer,
{
    pub fn init(mut self) -> io::Result<WavWriter<'a, T, Initiated>> {
        self.header_position = self.out_buffer.stream_position()?;
        let header: [u8; HEADER_SIZE] = self.wav_opts.borrow().into();
        // don't count in written_data
        self.out_buffer.write_all(&header)?;
        self.wav_opts.written_data = 0;
        Ok(self.transition())
    }
}

impl<'a, T: WavOutBuffer> WavWriter<'a, T, Initiated> {
    pub fn close(mut self) -> io::Result<()> {
        self.out_buffer.flush()?;
        println!("{:?}", self.wav_opts);
        let last_pos = self.out_buffer.stream_position()?;
        let offset: i64 = (last_pos - self.header_position).try_into().unwrap();
        self.out_buffer.seek(SeekFrom::Current(-offset))?;
        let header: [u8; HEADER_SIZE] = self.wav_opts.borrow().into();
        // don't count in written_data
        self.out_buffer.write_all(&header)?;
        self.out_buffer.flush()?;
        Ok(())
    }

    pub fn write_half_words(&mut self, data: &[i16]) -> io::Result<()> {
        for half_word in data {
            self.write_all(&(*half_word as u16).to_le_bytes())?;
        }
        Ok(())
    }
}

impl<'a, T: WavOutBuffer> Write for WavWriter<'a, T, Initiated> {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let bytes_written = self.out_buffer.write(data)?;
        if let Some(tot_data) = self.wav_opts.written_data.checked_add(bytes_written as u32) {
            self.wav_opts.written_data = tot_data;
        } else {
            panic!(
                "Wrote more than {} bytes of data, can't be represented in this file format",
                u32::MAX
            )
        }
        Ok(bytes_written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.out_buffer.flush()
    }
}

pub fn write_wav(data: Vec<i16>, sample_rate: u32, writer: &mut dyn Write) -> io::Result<()> {
    fn make_bytes<T>(number: T) -> Vec<u8>
    where
        T: Into<u32>,
    {
        let number: u32 = number.into();
        let mut b: Vec<u8> = Vec::new();
        for i in 0..std::mem::size_of::<T>() {
            b.push(((number >> (8 * i)) & 0xff) as u8);
        }
        b
    }
    let nsamples = data.len() * 2;
    writer.write_all(b"RIFF")?;
    let rsize = make_bytes::<u32>(20 + nsamples as u32); // added 20 for the rest of the header
    writer.write_all(&rsize)?; // WAVE chunk size

    // WAVE chunk
    writer.write_all(b"WAVE")?;

    // fmt chunk
    writer.write_all(b"fmt ")?;
    writer.write_all(&make_bytes::<u32>(16))?; // fmt chunk size
    writer.write_all(&make_bytes::<u16>(1))?; // format code (PCM)
    writer.write_all(&make_bytes::<u16>(1))?; // number of channels
    writer.write_all(&make_bytes::<u32>(sample_rate))?; // sample rate
    writer.write_all(&make_bytes::<u32>(sample_rate))?; // data rate
    writer.write_all(&make_bytes::<u16>(2))?; // block size
    writer.write_all(&make_bytes::<u16>(16))?; // bits per sample

    // data chunk
    writer.write_all(b"data")?;
    writer.write_all(&make_bytes::<u32>(nsamples as u32))?; // data chunk size
    for half_word in data {
        writer.write_all(&make_bytes(half_word as u16))?;
    }

    writer.flush()
}

#[test]
fn test_file() {
    use super::wav_writer::WavBuilder;
    use crate::polyphonia::{notable_notes, Amplitude, Note};
    use std::fs::OpenOptions;

    let mut out_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("file_static.wav")
        .unwrap();

    let wav_file = WavBuilder::new().set_output(&mut out_file);
    let mut wav_file = wav_file.init().unwrap();
    wav_file.write_all(&[0; 28]).unwrap();
    wav_file
        .write_half_words(&notable_notes::C4.audio_wave(3.0, &Amplitude::Medium))
        .unwrap();
    wav_file
        .write_half_words(&notable_notes::A4.audio_wave(3.0, &Amplitude::Silent))
        .unwrap();
    wav_file
        .write_half_words(&notable_notes::A4.audio_wave(3.0, &Amplitude::Low))
        .unwrap();
    wav_file
        .write_half_words(&Note::combine(
            &[notable_notes::A4, notable_notes::C4_SH, notable_notes::E0],
            3.0,
            &Amplitude::Medium,
        ))
        .unwrap();
    wav_file.close().unwrap()
}
