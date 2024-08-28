extern crate clap;
extern crate core;


use std::fs::File;
use std::io::{Read, Write};
use std::ops::Add;
use std::path::{Path, PathBuf};
use clap::{Arg, ArgMatches, App, SubCommand};
use crate::script_decoder::ScriptYsV;
use crate::unpacker::Ysv;
mod func;
mod unpacker;
mod script_decoder;

const OFFSETS:[u32;3] = [0x187A20,0x18E7D0,0x2369A0];


fn decode(matches: &ArgMatches){
    if matches.is_present("stagexx.bin"){
        let bin_name =Path::to_path_buf(matches.value_of("stagexx.bin").unwrap().as_ref());
        let mut script_file = File::open(&bin_name).expect("Script file not found");
        let mut buffer = Vec::new();
        script_file.read_to_end(&mut buffer).unwrap();
        let  script = ScriptYsV::new(&buffer);
        let json_name = Path::to_path_buf(matches.value_of("stagexx.bin").unwrap().to_lowercase().to_string().replace("bin", "json").as_ref());
        File::create(&json_name)
            .expect("Create json error!")
            .write((&serde_json::to_string_pretty(&script).unwrap()).as_ref())
            .unwrap();


        println!("Decode {} to {}", bin_name.to_str().unwrap().to_string(), json_name.to_str().unwrap().to_string());
    }
}
fn encode(matches: &ArgMatches){
    if matches.is_present("stagexx.json"){
        let json_name =Path::to_path_buf(matches.value_of("stagexx.json").unwrap().as_ref());
        let bin_name = Path::to_path_buf(matches.value_of("stagexx.json").unwrap().to_lowercase().to_string().replace("json", "bin").as_ref());
        let mut buffer = Vec::new();
        File::open(&json_name).expect("File json tidak ada!").read_to_end(&mut buffer).unwrap();
        let script_string = String::from_utf8(buffer).unwrap();
        let hj:ScriptYsV = serde_json::from_str(&script_string).unwrap();
        hj.to_bin(&bin_name);
        println!("Encode {} to {}",json_name.to_str().unwrap().to_string(),bin_name.to_str().unwrap().to_string());
    }


}

pub fn repack(matches: &ArgMatches){
    if matches.is_present("path"){
        let mut overwrite= false;
        if matches.is_present("overwrite"){
            overwrite = true;
        }
        let input_path =  PathBuf::from(matches.value_of("path").unwrap_or("./"));
        let elf_path = input_path.join("SLPM_663.60");
        let mut o = File::open(Path::to_path_buf(&elf_path.as_ref()).to_str().unwrap().to_string().add(".json")).expect("Json not found");
        let mut buffer = Vec::new();
        let _ = o.read_to_end(&mut buffer);
        let str = String::from_utf8(buffer).unwrap();
        let mut slpm_663_60:Ysv = serde_json::from_str(&str).unwrap();
        slpm_663_60.repack(elf_path, input_path, overwrite);
    }
    println!("Repack Selesai!!");

}
fn unpack(matches: &ArgMatches) {
    if matches.is_present("path") {
        let input_path =  PathBuf::from(matches.value_of("path").unwrap_or("./"));
        let elf_path = input_path.join("SLPM_663.60");
        let mut elf_file = File::open(&elf_path).expect("SLPM_663.60 not found");
        let mut buffer = Vec::new();
        elf_file.read_to_end(&mut buffer).unwrap();
        let  slpm_663_60 = Ysv::new(&buffer, OFFSETS);
        slpm_663_60.unpack(&input_path);
        let slpm_663_60_ser = serde_json::to_string_pretty(&slpm_663_60).unwrap();
        File::create(
            Path::to_path_buf(&elf_path)
                .to_str()
                .unwrap()
                .to_string()
                .add(".json"))
            .expect(" Write SLPM_663.60.json error!")
            .write((&slpm_663_60_ser).as_ref()).unwrap();
    }
    println!("Unpack Selesai!!");
}

fn main() {
    let app = App::new("Ys V Lost Kefin Archive unpacker/repacker")
        .version("1.0")
        .author("Gil Unx")
        .about("Unpack file dari .BIN  Ys V Lost Kefin  PS2 ISO")
        .subcommand(SubCommand::with_name("unpack")
            .arg(Arg::with_name("path")
                .short("p")
                .long("path")
                .help("Path to SLPM_663.60, DATA.BIN ,DATA0.BIN, DATA1.BIN files")
                .takes_value(true)
               )
        )
        .subcommand(SubCommand::with_name("repack")
            .arg(Arg::with_name("overwrite")
                .short("o")
                .long("overwrite")
                .help("Overwrite SLPM_663.60, DATA.BIN ,DATA0.BIN, DATA1.BIN")

            )
            .arg(Arg::with_name("path")
                .short("p")
                .long("path")
                .help("Path to SLPM_663.60, DATA.BIN ,DATA0.BIN, DATA1.BIN files")
                .takes_value(true)
                .index(1)

            )

        )
        .subcommand(SubCommand::with_name("decode")
            .arg(
                Arg::with_name("stagexx.bin")
                .help("decode stagexx.bin to stagexx.json")
                .takes_value(true)



            )
        )
        .subcommand(SubCommand::with_name("encode")
            .arg(
                Arg::with_name("stagexx.json")
                    .help("encode stagexx.json to stagexx.bin")
                    .takes_value(true)


            )
        )
        ;

    let matches = app.get_matches();

    match matches.subcommand() {
        ("unpack", Some(m)) => unpack(&m),
        ("repack", Some(m)) => repack(&m),
        ("decode", Some(m)) => decode(&m),
        ("encode", Some(m)) => encode(&m),
        _ => println!("{}", matches.usage()),
    };




}
