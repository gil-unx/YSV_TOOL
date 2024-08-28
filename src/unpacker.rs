use std::fs;
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::ops::Add;
use std::path::{Path, PathBuf};
use binread::BinReaderExt;
use binread::io::StreamPosition;
use serde::{Deserialize, Serialize};
use crate::func::{CursorHelper, file_load};

#[derive(Debug,Serialize, Deserialize)]
struct Folder {
    name_offset: u32,
    name:String,
    index:u32,
    file_count:u32,
    files:Vec<DataFile>,

}
#[derive(Debug,Serialize, Deserialize)]
struct DataFile{
    name_offset:u32,
    name:String,
    unk:u32,
    offset:u32,
    size:u32
}
#[derive(Debug,Serialize, Deserialize)]
struct DataInfo{
    name:String,
    name_offset:u32,
    folder_count:u32,
    folders_offset:u32,
    unk:u32,
    file_count:u32,
    files_offset:u32,
    folders:Vec<Folder>,

}
impl DataInfo {
    pub fn new(buffer:&[u8],offset:u32)->Self{
        let mut reader = Cursor::new(buffer);
        reader.seek(SeekFrom::Start(offset as u64)).unwrap();
        let name_offset =  reader.read_le::<u32>().unwrap() -0xFFF80;
        let folder_count =  reader.read_le::<u32>().unwrap();
        let folders_offset =  reader.read_le::<u32>().unwrap()-0xFFF80;
        let unk =  reader.read_le::<u32>().unwrap();
        let file_count =  reader.read_le::<u32>().unwrap();
        let files_offset =  reader.read_le::<u32>().unwrap()-0xFFF80;
        let mut folders:Vec<Folder> = Vec::new();
        let name = reader.get_shift_jis(name_offset as u64);
        reader.seek(SeekFrom::Start(folders_offset as u64)).unwrap();
        for _ in 0..folder_count {
            let folder_name_offset = reader.read_le::<u32>().unwrap() -0xFFF80;
            let index= reader.read_le::<u32>().unwrap();
            let file_count= reader.read_le::<u32>().unwrap();
            let name=reader.get_shift_jis(folder_name_offset as u64);
            let files=Vec::new();
            folders.push(
                Folder {
                    name_offset,
                    name,
                    index,
                    file_count,
                    files,
                }
            );
        }
        for folder in &mut folders {
            let mut data_files:Vec<DataFile> = Vec::new();
            for i in 0..folder.file_count {
                reader.seek(SeekFrom::Start(files_offset as u64 +((folder.index+i)*0x10) as u64)).unwrap();
                let name_offset=reader.read_le::<u32>().unwrap()-0xFFF80;
                let name=reader.get_shift_jis(name_offset as u64);
                let unk=reader.read_le::<u32>().unwrap();
                let offset=reader.read_le::<u32>().unwrap()*0x800;
                let size=reader.read_le::<u32>().unwrap()*0x800;
                data_files.push(
                    DataFile{
                        name_offset,
                        name,
                        unk,
                        offset,
                        size,
                    }
                )
            }

            folder.files = data_files;
        }

        let data_info = Self {
            name_offset,
            folder_count,
            folders_offset,
            unk,
            file_count,
            files_offset,
            name,
            folders
        };
        data_info
    }

}
#[derive(Debug,Serialize, Deserialize)]
pub struct Ysv {
    data_list:Vec<DataInfo>,
}
impl Ysv {
    pub fn new(buffer: &Vec<u8>, offsets: [u32;3]) ->Self{
        let mut data_list :Vec<DataInfo> = Vec::new();
        for offset in offsets {
            data_list.push(DataInfo::new(&buffer, offset));
        }
        let  meta = Self {
            data_list
        };
        meta
    }
    pub fn unpack(&self, input_folder: &PathBuf){
        for data in &self.data_list {
            let data_name = input_folder.join(&data.name);
            let mut file_stream = File::open(&data_name).expect("DATA*.BIN not found");
            let out_folder = Path::parent(&data_name).unwrap().join(Path::file_stem(data.name.as_ref()).unwrap().to_str().unwrap().to_string()) ;
            fs::create_dir_all(out_folder.clone()).unwrap();
            for folder in &data.folders {
                fs::create_dir_all( PathBuf::from(out_folder.to_str().unwrap().to_string().add(&folder.name))).unwrap() ;
                for file in &folder.files {
                    let output_file_name = PathBuf::from(out_folder.to_str().unwrap().to_string().add(&folder.name).add(&file.name));
                    println!("Unpack >>{}",output_file_name.to_str().unwrap());
                    let  buffer =file_stream.get_bytes(file.offset as u64,file.size as usize);
                    File::create(output_file_name).unwrap().write(&buffer).unwrap();


                }
            }
        }
    }
    pub fn repack(&mut self, elf_name:PathBuf, input_folder: PathBuf, overwrite:bool){
        let mut elf_buffer = Vec::new();
        File::open(&elf_name).expect("SLPM not found").read_to_end(&mut elf_buffer).unwrap();
        let mut elf_output  = elf_name.clone();
        let mut elf_stream = Cursor::new(elf_buffer);
        for data in &mut self.data_list {
            let mut  data_name = input_folder.join(&data.name);
            if !overwrite{
                data_name  = Path::file_stem(data.name.as_ref()).unwrap().to_str().unwrap().to_string().add("_NEW.BIN").parse::<PathBuf>().unwrap();
            }
            let mut file_stream = File::create(input_folder.join(&data_name)).expect("Create BIN error!");
            let out_folder = input_folder.join(Path::file_stem((&data.name).as_ref()).unwrap().to_str().unwrap().to_string());
            elf_stream.seek(SeekFrom::Start(data.files_offset as u64)).unwrap();
            let mut new_offset:u32;
            for folder  in  &data.folders{
                for file in &folder.files {
                    new_offset = (file_stream.stream_pos().unwrap()/0x800)as u32  ;
                    let input_file_name = PathBuf::from(out_folder.to_str().unwrap().to_string().add(&folder.name).add(&file.name));
                    let buffer = file_load(&input_file_name);
                    let new_size = buffer.len()/0x800;
                    elf_stream.write(&(file.name_offset + 0xFFF80u32).to_le_bytes()).unwrap();
                    elf_stream.write(&file.unk.to_le_bytes()).unwrap();
                    elf_stream.write(&new_offset.to_le_bytes()).unwrap();
                    elf_stream.write(&(new_size as u32).to_le_bytes()).unwrap();
                    file_stream.write(&buffer).unwrap();
                    println!("Repack >>{}",&input_file_name.to_str().unwrap().to_string());

                }
            }
        }
        if !overwrite{
            elf_output  = Path::file_stem(&elf_name).unwrap().to_str().unwrap().to_string().add("_NEW.60").parse::<PathBuf>().unwrap();
        }
        elf_stream.seek(SeekFrom::Start(0)).unwrap();
        let mut new_elf_buffer = Vec::new();
        elf_stream.read_to_end(&mut new_elf_buffer).unwrap();
        File::create(&elf_output).expect("Create SLPM error").write(&new_elf_buffer).unwrap();

    }
}
