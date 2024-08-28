use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use binread::BinReaderExt;
use binread::io::StreamPosition;

extern crate encoding_rs;
use encoding_rs::SHIFT_JIS;

pub fn file_load(file_name: &PathBuf) ->Vec<u8>{
    let mut f = File::open(file_name).expect("File not found");
    let mut buffer = Vec::new();
    let _ = f.read_to_end(&mut buffer);
    buffer

}
pub trait CursorHelper {
    fn get_bytes(&mut self,offset:u64,size:usize)->Vec<u8>;
    fn get_shift_jis(&mut self,offset:u64)->String;
    fn read_fixed_shift_jis(&mut self,len:usize)->String;
    fn read_shift_jis(&mut self,)->String;
    fn write_padding(&mut self,padding:u64,pos:u64);
    fn read_u8(&mut self)->u8;
    fn read_u16(&mut self)->u16;
    fn read_u32(&mut self)->u32;


}
pub trait WriterHelper<T>{
    fn write_v(&mut self,value: T);

}
impl WriterHelper<u32> for Cursor<&mut Vec<u8>>{
    fn write_v(&mut self, value: u32) {
        self.write(&value.to_le_bytes()).unwrap();
    }
}
impl WriterHelper<&u32> for Cursor<&mut Vec<u8>>{
    fn write_v(&mut self, value: &u32) {
    self.write(&value.to_le_bytes()).unwrap();
}
}
impl WriterHelper<u16> for Cursor<&mut Vec<u8>>{
    fn write_v(&mut self, value: u16) {
        self.write(&value.to_le_bytes()).unwrap();
    }
}
impl WriterHelper<&u16> for Cursor<&mut Vec<u8>>{
    fn write_v(&mut self, value: &u16) {
        self.write(&value.to_le_bytes()).unwrap();
    }
}
impl WriterHelper<u8> for Cursor<&mut Vec<u8>>{
    fn write_v(&mut self, value: u8) {
        self.write(&value.to_le_bytes()).unwrap();
    }
}
impl WriterHelper<&u8> for Cursor<&mut Vec<u8>>{
    fn write_v(&mut self, value: &u8) {
        self.write(&value.to_le_bytes()).unwrap();
    }
}
impl CursorHelper for File {
    fn get_bytes(&mut self, offset: u64, size: usize) -> Vec<u8> {
        let current_offset = self.stream_position().unwrap();
        let mut bytes:Vec<u8> = vec![0u8; size];
        self.seek(SeekFrom::Start(offset)).expect("Seek error!!");
        self.read_exact(&mut bytes).expect("Read bytes error");
        self.seek(SeekFrom::Start(current_offset)).expect("Seek error!!");
        bytes
    }

    fn get_shift_jis(&mut self, offset: u64) -> String {
        let current_offset = self.stream_position().unwrap();
        self.seek(SeekFrom::Start(offset)).expect("Seek error!!");
        let mut  b :Vec<u8> = Vec::new();
        loop {
            let char =  self.read_le::<u8>().unwrap();
            if char ==0{
                break;
            }
            b.push(char);
        }
        let (res, _enc, _errors) = SHIFT_JIS.decode(&b);
        self.seek(SeekFrom::Start(current_offset)).expect("Seek error!!");
        res.to_string()
    }

    fn read_fixed_shift_jis(&mut self, len: usize) -> String {
        let mut  b :Vec<u8> = Vec::new();
        for _ in 0..len {
            b.push(self.read_le::<u8>().unwrap());
        }
        let (res, _enc, _errors) = SHIFT_JIS.decode(&b);
        res.to_string()
    }

    fn read_shift_jis(&mut self) -> String {
        let mut  b :Vec<u8> = Vec::new();
        loop {
            let char =  self.read_le::<u8>().unwrap();
            if char ==0{
                break;
            }
            b.push(char);
        }
        let (res, _enc, _errors) = SHIFT_JIS.decode(&b);

        res.to_string()
    }

    fn write_padding(&mut self, padding: u64, pos: u64) {
        loop {
            if (self.stream_pos().unwrap() - pos) % padding as u64 == 0{
                break;
            }
            self.write(&0u8.to_le_bytes()).unwrap();
        }
    }

    fn read_u8(&mut self) -> u8 {
        self.read_le::<u8>().unwrap()
    }

    fn read_u16(&mut self) -> u16 {
        self.read_le::<u16>().unwrap()
    }

    fn read_u32(&mut self) -> u32 {
        self.read_le::<u32>().unwrap()
    }
}

impl CursorHelper for Cursor<&[u8]> {
    fn get_bytes(&mut self, offset: u64, size: usize) -> Vec<u8> {
        let current_offset = self.stream_position().unwrap();
        let mut bytes:Vec<u8> = vec![0u8; size];
        self.seek(SeekFrom::Start(offset)).expect("Seek error!!");
        self.read_exact(&mut bytes).expect("Read bytes error");
        self.seek(SeekFrom::Start(current_offset as u64)).expect("Seek error!!");
        bytes
    }

    fn get_shift_jis(&mut self, offset: u64) -> String {
        let current_offset = self.stream_position().unwrap();
        self.seek(SeekFrom::Start(offset)).expect("Seek error!!");
        let mut  b :Vec<u8> = Vec::new();
        loop {
            let char =  self.read_le::<u8>().unwrap();
            if char ==0{
                break;
            }
            b.push(char);
        }
        let (res, _enc, _errors) = SHIFT_JIS.decode(&b);
        self.seek(SeekFrom::Start(current_offset)).expect("Seek error!!");
        res.to_string()
    }

    fn read_fixed_shift_jis(&mut self, len: usize) -> String {
        let mut tmp :Vec<u8> = vec![0u8;len];

        let _ = self.read_exact(&mut tmp);
        tmp.push(0);
        tmp.push(0);
        tmp.push(0);
        let mut  b :Vec<u8> = Vec::new();
        let mut n =0;
        loop {
            if (tmp[n] == 0xff) &&(tmp[n+1] == 0xfd){
                let ctrl = format!("{:02X}",tmp[n+2]).into_bytes();
                b.push(0x5B);
                b.push(0x7B);
                b.push(0x46);
                b.push(0x44);
                b.push(0x46);
                b.push(0x46);
                b.push(ctrl[0]);
                b.push(ctrl[1]);
                b.push(0x7D);
                b.push(0x5D);
                n+=3;
            }
            else{
                let c = tmp[n];
                b.push(c);
                n+=1
            }
            if n >= len{
                break;
            }
        }

        let (res, _enc, _errors) = SHIFT_JIS.decode(&b);
        res.to_string()
    }

    fn read_shift_jis(&mut self) -> String {
        let mut  b :Vec<u8> = Vec::new();
        loop {
            let char =  self.read_le::<u8>().unwrap();
            if char ==0{
                break;
            }
            b.push(char);
        }
        let (res, _enc, _errors) = SHIFT_JIS.decode(&b);

        res.to_string()
    }

    fn write_padding(&mut self, _padding: u64, _pos: u64) {
        todo!()
    }

    fn read_u8(&mut self) -> u8 {
        self.read_le::<u8>().unwrap()
    }

    fn read_u16(&mut self) -> u16 {
        self.read_le::<u16>().unwrap()
    }

    fn read_u32(&mut self) -> u32 {
        self.read_le::<u32>().unwrap()
    }
}
