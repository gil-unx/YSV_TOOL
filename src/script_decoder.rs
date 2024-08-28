
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom, stdin, Write};
use std::path::PathBuf;
use binread::io::StreamPosition;
use encoding_rs::SHIFT_JIS;
use serde::{Deserialize, Serialize};

use crate::func::{CursorHelper, WriterHelper};

const REFERENCE:u64 =0x2000;
#[derive(Debug,Serialize, Deserialize)]
struct StartEnd{
    start:u32,
    end:u32
}
#[derive(Debug,Serialize, Deserialize)]
struct Choice{
    len:u16,
    choice:String,
    end_0:u16,
    end_1:u16,
}
#[derive(Debug,Serialize, Deserialize)]
enum Code{
    C00{},
    C01{ a0:u8},
    C02{ a0:u8},
    C03{ a0:u8},
    C04{},
    C05{},
    C06{},
    C07{ a0:u8},
    C08{ xstr:String},
    C09{ xstr:String},
    C0A{ a0:u16, a1:u16, len:u16, str:String},
    C0B{},
    C0C{},
    C0D{},
    C0E{ a0:u8},
    C0F{},
    C10{},
    C11{},
    C12{},
    C13{ a0:u16,a1:u8},
    C14{ a0:u8},
    C15{ a0:u16},
    C16{ a0:u8,a1:u8,a2:u8},
    C17{ str:String,a0:u16},
    C18{ a0:u8,a1:u8,a2:u8},
    C19{ a0:u16,a1:u16,a2:u16,a3:u16},
    C1A{ a0:u16,a1:u8},
    C1B{ a0:u16,a1:u32},
    C1C{ a0:u16,a1:u16},
    C1D{ a0:u16,a1:u8},
    C1E{ a0:u16,a1:u16,a2:u16,a3:u16,a4:u8},
    C1F{ a0:u16,a1:u8,a2:u8},
    C20{ a0:u16,a1:u8},
    C21{ a0:u16,a1:u8},
    C22{ a0:u16,a1:u16},
    C23{ a0:u8,a1:u32},
    C24{ a0:u16, a1:u16, len:u16, ask:String, choices:Vec<Choice>},
    C25{ a0:u16, len:u16, ask:String, choices:Vec<Choice>},
    C26{ a0:u16, len:u16, ask:String, choices:Vec<Choice>},
    C27{ a0:u16, a1:u16, len:u16, ask:String, choices:Vec<Choice>},
    C28{ a0:u16, len:u16, ask:String, choices:Vec<Choice>},
    C29{ a0:u16, len:u16, ask:String, choices:Vec<Choice>},
    C2A{ a0:u16,a1:u8},
    C2B{ a0:u8},
    C2C{ a0:u8},
    C2D{ a0:u16,a1:u16,a2:u16,a3:u16,a4:u8},
    C2E{ xstr:String},
    C2F{ a0:u16},
    C30{ a0:u8},
    C31{ a0:u8},
    C32{ a0:u8},
    C33{ a0:u8},
    C34{},
    C35{ a0:u16, a1:u16, len:u16, str:String, },
    C36{ a0:u16, a1:u16, a2:u16, a3:u16, a4:u16},
    C37{ a0:u16, a1:u16, a2:u16, a3:u16, a4:u16},
    C38{ a0:u16, a1:u16, a2:u16},
    C39{ a0:u16, a1:u16, a2:u16},
    C3A{ a0:u16, a1:u16},
    C3B{ a0:u16, len:u16, str:String },
    C3C{ jump:u16,a1:u16},
    C3D{ a0:u8 },
    C3E{ a0:u8 },
    C3F{ a0:u8 },
    C40{ a0:u8 },
    C41{ a0:u8 },
    C42{ a0:u16 },
    C43{ a0:u16 },
    C44{ a0:u16,a1:u16},
    C45{ a0:u8 },
    C46{ },
    C47{ a0:u8 },
    C48{ a0:u8},
    C49{ a0:u16},
    C4A{ a0:u16,a1:u8},
    C4B{ a0:u16,a1:u8},
    C4C{ a0:u8},
    C4D{ a0:u8},
    C4E{ },
    C4F{ a0:u16},
    C50{ a0:u16,a1:u32},
    C51{ },
    C52{ a0:u16, len:u16, str:String },
    C53{ a0:u16},
    C54{ xstr:String},
    C55{ },
    C56{ a0:u8},
    C57{ a0:u16},
    C58{ a0:u16},
    C59{ a0:u16},
    C5A{},
    C5B{a0:u16,a1:u8},
    C5C{a0:u16,a1:u8},
    C5D{a0:u16,a1:u8},
    C5E{a0:u8},
    C5F{a0:u8},
    C60{a0:u16},
    C61{a0:u8},
    C62{a0:u16,a1:u8},
    C63{a0:u16,a1:u8},
    C64{a0:u16,a1:u16,a2:u16,a3:u16,a4:u16,a5:u16,a6:u8},
    C65{a0:u16},
    C66{a0:u8,a1:u32,a2:u32},
    C67{},
    End{}


}
#[derive(Debug,Serialize, Deserialize)]
struct ListCodes {

    codes:Vec<Code>
}
fn get_bytes_string(str: &String) ->Vec<u8>{
    let (bytes_tmp_string,_,_) = SHIFT_JIS.encode(&str);
    let mut bytes_string = Vec::new();
    let mut  n =0;
    loop {
        if (bytes_tmp_string[n] == 0x5b) &&(bytes_tmp_string[n+1] == 0x7b){

            bytes_string.push(0xFF);
            bytes_string.push(0xFD);
            let mut ctrl:Vec<u8> =Vec::new();
            ctrl.push(bytes_tmp_string[n+6]);
            ctrl.push(bytes_tmp_string[n+7]);
            let value = u8::from_str_radix(&String::from_utf8(ctrl).unwrap(),16).unwrap();
            bytes_string.push(value);
            n+=10;
        }
        else{

            bytes_string.push(bytes_tmp_string[n]);
            n+=1;
        }
        if n >= bytes_tmp_string.len(){
            break;
        }
    }
    bytes_string

}
fn decode(reader: &mut Cursor<&[u8]>) -> (Vec<Code>, Vec<u32>, Vec<u32>) {
    let mut codes:Vec<Code> = Vec::new();
    let mut start_3c:Vec<u32> = Vec::new();
    let mut end_3c:Vec<u32> = Vec::new();
    let mut s = String::new();
    let mut pos:u64;
    let mut pos_end = 0xFFFFFFFFFFFFFFFF;
    let mut n:u32 = 0;
    loop {
        pos = reader.stream_pos().unwrap();
        if pos == pos_end{
            end_3c.push(n);
        }
        let b = reader.read_u8();
        match b {
            0x00 =>{codes.push(Code::C00{});}
            0x01 =>{codes.push(Code::C01{a0: reader.read_u8()});}
            0x02 =>{codes.push(Code::C02{a0: reader.read_u8()});}
            0x03 =>{codes.push(Code::C03{a0: reader.read_u8()});}
            0x04 =>{codes.push(Code::C04{});}
            0x05 =>{codes.push(Code::C05{});}
            0x06 =>{codes.push(Code::C06{});}
            0x07 =>{codes.push(Code::C07{a0: reader.read_u8()});}
            0x08 =>{
                start_3c.clear();
                end_3c.clear();
                codes.push(
                    Code::C08{
                        xstr:reader.read_shift_jis()

                    });

             }//string
            0x09 =>{
                start_3c.clear();
                end_3c.clear();
                codes.push(
                    Code::C09{
                        xstr:reader.read_shift_jis()

                });

             }//string
            0x0A =>{
                let a0= reader.read_u16();
                let a1= reader.read_u16();
                let len= reader.read_u16()  & 0x7fff;
                let str = reader.read_fixed_shift_jis(len as usize);
                codes.push(
                    Code::C0A{
                        a0,
                        a1,
                        len,
                        str
                    }
                );

            }//string
            0x0B =>{codes.push(Code::C0B{});}
            0x0C =>{codes.push(Code::C0C{});}
            0x0D =>{codes.push(Code::C0D{});}
            0x0E =>{codes.push(Code::C0E{a0: reader.read_u8()});}
            0x0F =>{codes.push(Code::C0F{});}
            0x10 =>{codes.push(Code::C10{});}
            0x11 =>{codes.push(Code::C11{});}
            0x12 =>{codes.push(Code::C12{});}
            0x13 =>{codes.push(Code::C13{a0: reader.read_u16(),a1: reader.read_u8()});}
            0x14 =>{codes.push(Code::C14{a0: reader.read_u8()});}
            0x15 =>{codes.push(Code::C15{a0: reader.read_u16()});}
            0x16 =>{codes.push(Code::C16{a0: reader.read_u8(),a1: reader.read_u8(),a2: reader.read_u8()});}
            0x17 =>{
                codes.push(
                    Code::C17{
                        str:reader.read_shift_jis(),
                        a0:reader.read_u16()
                    }
                );

            }//string
            0x18 =>{codes.push(Code::C18{a0: reader.read_u8(),a1: reader.read_u8(),a2: reader.read_u8()});}
            0x19 =>{codes.push(Code::C19{a0: reader.read_u16(),a1: reader.read_u16(),a2: reader.read_u16(),a3: reader.read_u16()});}
            0x1A =>{codes.push(Code::C1A{a0: reader.read_u16(),a1: reader.read_u8()});}
            0x1B =>{codes.push(Code::C1B{a0: reader.read_u16(),a1: reader.read_u32()});}
            0x1C =>{codes.push(Code::C1C{a0: reader.read_u16(),a1: reader.read_u16()});}
            0x1D =>{codes.push(Code::C1D{a0: reader.read_u16(),a1: reader.read_u8()});}
            0x1E =>{codes.push(Code::C1E{a0: reader.read_u16(),a1: reader.read_u16(),a2: reader.read_u16(),a3: reader.read_u16(),a4: reader.read_u8()});}
            0x1F =>{codes.push(Code::C1F{
                a0: reader.read_u16(),
                a1: reader.read_u8(),
                a2: reader.read_u8()
            });}
            0x20 =>{codes.push(Code::C20{a0: reader.read_u16(),a1: reader.read_u8()});}
            0x21 =>{codes.push(Code::C21{a0: reader.read_u16(),a1: reader.read_u8()});}
            0x22 =>{codes.push(Code::C22{a0: reader.read_u16(),a1: reader.read_u16()});}
            0x23 =>{codes.push(Code::C23{a0: reader.read_u8(),a1: reader.read_u32()});}
            0x24 =>{
                let a0= reader.read_u16();
                let a1= reader.read_u16();
                let len= reader.read_u16()  & 0x7fff;
                let ask = reader.read_fixed_shift_jis(len as usize);
                let mut choices:Vec<Choice> = Vec::new();
                for _ in 0..4 {
                    let lenc= reader.read_u16()  & 0x7fff;
                    let choice = reader.read_fixed_shift_jis(lenc as usize);
                    let end_0c=reader.read_u16();
                    let end_1c=reader.read_u16();
                    choices.push(
                        Choice{
                            len:lenc,
                            choice,
                            end_0:end_0c,
                            end_1:end_1c

                        }
                    )
                }
                codes.push(
                    Code::C24{
                        a0,
                        a1,
                        len,
                        ask,
                        choices
                    }
                );
            }//string
            0x25 =>{
                let a0= reader.read_u16();
                let len= reader.read_u16()  & 0x7fff;
                let ask = reader.read_fixed_shift_jis(len as usize);
                let mut choices:Vec<Choice> = Vec::new();
                for _ in 0..4 {
                    let lenc= reader.read_u16()  & 0x7fff;
                    let choice = reader.read_fixed_shift_jis(lenc as usize);
                    let end_0c=reader.read_u16();
                    let end_1c=reader.read_u16();
                    choices.push(
                        Choice{
                            len:lenc,
                            choice,
                            end_0:end_0c,
                            end_1:end_1c

                        }
                    )
                }
                codes.push(
                    Code::C25{
                        a0,
                        len,
                        ask,
                        choices
                    }
                );

            }//string
            0x26 =>{
                let a0= reader.read_u16();
                let len= reader.read_u16()  & 0x7fff;
                let ask = reader.read_fixed_shift_jis(len as usize);
                let mut choices:Vec<Choice> = Vec::new();
                for _ in 0..4 {
                    let lenc= reader.read_u16()  & 0x7fff;
                    let choice = reader.read_fixed_shift_jis(lenc as usize);
                    let end_0c=reader.read_u16();
                    let end_1c=reader.read_u16();
                    choices.push(
                        Choice{
                            len:lenc,
                            choice,
                            end_0:end_0c,
                            end_1:end_1c

                        }
                    )
                }
                codes.push(
                    Code::C26{
                        a0,
                        len,
                        ask,
                        choices
                    }
                );
            }//string
            0x27 =>{
                let a0= reader.read_u16();
                let a1= reader.read_u16();
                let len= reader.read_u16()  & 0x7fff;
                let ask = reader.read_fixed_shift_jis(len as usize);
                let mut choices:Vec<Choice> = Vec::new();
                for _ in 0..2 {
                    let lenc= reader.read_u16()  & 0x7fff;
                    let choice = reader.read_fixed_shift_jis(lenc as usize);
                    let end_0c=reader.read_u16();
                    let end_1c=reader.read_u16();
                    choices.push(
                        Choice{
                            len:lenc,
                            choice,
                            end_0:end_0c,
                            end_1:end_1c

                        }
                    )
                }
                codes.push(
                    Code::C27{
                        a0,
                        a1,
                        len,
                        ask,
                        choices
                    }
                );
            }//string
            0x28 =>{//choice
                let a0= reader.read_u16();
                let len= reader.read_u16()  & 0x7fff;
                let ask = reader.read_fixed_shift_jis(len as usize);
                let mut choices:Vec<Choice> = Vec::new();
                for _ in 0..2 {
                    let lenc= reader.read_u16()  & 0x7fff;
                    let choice = reader.read_fixed_shift_jis(lenc as usize);
                    let end_0=reader.read_u16();
                    let end_1=reader.read_u16();
                    choices.push(
                        Choice{
                            len:lenc,
                            choice,
                            end_0,
                            end_1

                    }
                    )
                }
                codes.push(
                    Code::C28{
                        a0,
                        len,
                        ask,
                        choices
                    }
                );
            }//string
            0x29 =>{
                let a0= reader.read_u16();
                let len= reader.read_u16()  & 0x7fff;
                let ask = reader.read_fixed_shift_jis(len as usize);
                let mut choices:Vec<Choice> = Vec::new();
                for _ in 0..2 {
                    let lenc= reader.read_u16()  & 0x7fff;
                    let choice = reader.read_fixed_shift_jis(lenc as usize);
                    let end_0c=reader.read_u16();
                    let end_1c=reader.read_u16();
                    choices.push(
                        Choice{
                            len:lenc,
                            choice,
                            end_0:end_0c,
                            end_1:end_1c

                        }
                    )
                }
                codes.push(
                    Code::C29{
                        a0,
                        len,
                        ask,
                        choices
                    }
                );

             }//string
            0x2A =>{codes.push(Code::C2A{a0: reader.read_u16(),a1: reader.read_u8()});}
            0x2B =>{codes.push(Code::C2B{a0: reader.read_u8()});}
            0x2C =>{codes.push(Code::C2C{a0: reader.read_u8()});}
            0x2D =>{codes.push(Code::C2D{a0: reader.read_u16(),a1: reader.read_u16(),a2: reader.read_u16(),a3: reader.read_u16(),a4: reader.read_u8()});}
            0x2E =>{
                start_3c.clear();
                end_3c.clear();
                codes.push(
                    Code::C2E{
                        xstr:reader.read_shift_jis()
                    });
             }//string
            0x2F =>{codes.push(Code::C2F{a0: reader.read_u16()});}
            0x30 =>{codes.push(Code::C30{a0: reader.read_u8()});}
            0x31 =>{codes.push(Code::C31{a0: reader.read_u8()});}
            0x32 =>{codes.push(Code::C32{a0: reader.read_u8()});}
            0x33 =>{codes.push(Code::C33{a0: reader.read_u8()});}
            0x34 =>{codes.push(Code::C34{});}
            0x35 =>{
                let a0= reader.read_u16();
                let a1= reader.read_u16();
                let len= reader.read_u16()  & 0x7fff;
                let str = reader.read_fixed_shift_jis(len as usize);
                codes.push(
                    Code::C35{
                        a0,
                        a1,
                        len,
                        str
                    }
                );

            }//string fix
            0x36 =>{codes.push(Code::C36{a0: reader.read_u16(),a1: reader.read_u16(),a2: reader.read_u16(),a3: reader.read_u16(),a4: reader.read_u16()});}
            0x37 =>{codes.push(Code::C37{a0: reader.read_u16(),a1: reader.read_u16(),a2: reader.read_u16(),a3: reader.read_u16(),a4: reader.read_u16()});}
            0x38 =>{codes.push(Code::C38{a0: reader.read_u16(),a1: reader.read_u16(),a2: reader.read_u16()});}
            0x39 =>{codes.push(Code::C39{a0: reader.read_u16(),a1: reader.read_u16(),a2: reader.read_u16()});}
            0x3A =>{codes.push(Code::C3A{a0: reader.read_u16(),a1: reader.read_u16()});}
            0x3B =>{
                let a0= reader.read_u16();
                let len= reader.read_u16()  & 0x7fff;
                let str = reader.read_fixed_shift_jis(len as usize);
                codes.push(
                    Code::C3B{
                        a0,
                        len,
                        str
                    }
                );

            }//string fix
            0x3C =>{
                start_3c.push(n);
                let a0=reader.read_u16();
                let a1=  reader.read_u16();
                pos_end = a0 as u64+ pos+1;
                codes.push(Code::C3C{
                    jump: a0,
                    a1
                }
                );
            }
            0x3D =>{codes.push(Code::C3D{a0: reader.read_u8()});}//stdin().read_line(&mut s).expect("sss");
            0x3E =>{codes.push(Code::C3E{a0: reader.read_u8()});}
            0x3F =>{codes.push(Code::C3F{a0: reader.read_u8()});}
            0x40 =>{codes.push(Code::C40{a0: reader.read_u8()});}
            0x41 =>{codes.push(Code::C41{a0: reader.read_u8()});}
            0x42 =>{codes.push(Code::C42{a0: reader.read_u16()});}
            0x43 =>{codes.push(Code::C43{a0: reader.read_u16()});}
            0x44 =>{codes.push(Code::C44{a0: reader.read_u16(),a1: reader.read_u16()});}
            0x45 =>{codes.push(Code::C45{a0: reader.read_u8()});}
            0x46 =>{codes.push(Code::C46{});}
            0x47 =>{codes.push(Code::C47{a0: reader.read_u8()});}
            0x48 =>{codes.push(Code::C48{a0: reader.read_u8()});}
            0x49 =>{codes.push(Code::C49{a0: reader.read_u16()});}
            0x4A =>{codes.push(Code::C4A{a0: reader.read_u16(),a1: reader.read_u8()});}
            0x4B =>{codes.push(Code::C4B{a0: reader.read_u16(),a1: reader.read_u8()});}
            0x4C =>{codes.push(Code::C4C{a0: reader.read_u8()});}
            0x4D =>{codes.push(Code::C4D{a0: reader.read_u8()});}
            0x4E =>{codes.push(Code::C4E{});}
            0x4F =>{codes.push(Code::C4F{a0: reader.read_u16()});}
            0x50 =>{codes.push(Code::C50{a0: reader.read_u16(),a1: reader.read_u32()});}
            0x51 =>{codes.push(Code::C51{});}
            0x52 =>{
                let a0= reader.read_u16();
                let len= reader.read_u16()  & 0x7fff;
                let str = reader.read_fixed_shift_jis(len as usize);
                codes.push(
                    Code::C52{
                        a0,
                        len,
                        str
                    }
                );
            }//string fix
            0x53 =>{codes.push(Code::C53{a0: reader.read_u16()});}
            0x54 =>{
                start_3c.clear();
                end_3c.clear();
                codes.push(
                    Code::C54{
                        xstr:reader.read_shift_jis()

                    });

            }//string fix
            0x55 =>{codes.push(Code::C55{});}
            0x56 =>{codes.push(Code::C56{a0: reader.read_u8()});}
            0x57 =>{codes.push(Code::C57{a0: reader.read_u16()});}
            0x58 =>{codes.push(Code::C58{a0: reader.read_u16()});}
            0x59 =>{codes.push(Code::C59{a0: reader.read_u16()});}
            0x5A =>{codes.push(Code::C5A{});}
            0x5B =>{codes.push(Code::C5B{a0: reader.read_u16(),a1: reader.read_u8()});}
            0x5C =>{codes.push(Code::C5C{a0: reader.read_u16(),a1: reader.read_u8()});}
            0x5D =>{codes.push(Code::C5D{a0: reader.read_u16(),a1: reader.read_u8()});}
            0x5E =>{codes.push(Code::C5E{a0: reader.read_u8()});}
            0x5F =>{codes.push(Code::C5F{a0: reader.read_u8()});}
            0x60 =>{codes.push(Code::C60{a0: reader.read_u16()});}
            0x61 =>{codes.push(Code::C61{a0: reader.read_u8()});}
            0x62 =>{codes.push(Code::C62{a0: reader.read_u16(),a1: reader.read_u8()});}
            0x63 =>{codes.push(Code::C63{a0: reader.read_u16(),a1: reader.read_u8()});}
            0x64 =>{codes.push(Code::C64{a0: reader.read_u16(),a1: reader.read_u16(),a2: reader.read_u16(),a3: reader.read_u16(),a4: reader.read_u16(),a5: reader.read_u16(),a6: reader.read_u8()})}
            0x65 =>{codes.push(Code::C65{a0: reader.read_u16()});}
            0x66 =>{codes.push(Code::C66{a0: reader.read_u8(),a1: reader.read_u32(),a2: reader.read_u32()});}
            0x67 =>{codes.push(Code::C67{});}
            0xff =>{codes.push(Code::End{});break;}
            _ =>{
                println!("unhacdled code{:08X} {:02X}",reader.position(),b);
                stdin().read_line(&mut s).expect("sss");


            }

        }
        n+=1;

    }
    (codes,start_3c,end_3c)
}

#[derive(Debug,Serialize, Deserialize)]
struct Content{
    idx:u32,
    offset:u32,
    codes:Vec<Code>,
    start_3c:Vec<u32>,
    end_3c:Vec<u32>

}

#[derive(Debug,Serialize, Deserialize)]
pub struct ScriptYsV{
    contents:Vec<Content>

}
impl ScriptYsV{
    pub fn new(buffer:&[u8])->Self{

        let mut reader = Cursor::new(buffer);
        let mut pointers :Vec<Content> = Vec::new();
        loop {
            let idx = reader.read_u32();
            let offset= reader.read_u32();
            let codes:Vec<Code> = Vec::new();
            let start_3c:Vec<u32>=Vec::new();
            let end_3c:Vec<u32>=Vec::new();
            let pointer = Content{
                idx,
                offset,
                codes,
                start_3c,
                end_3c
            };

            if (pointer.idx == 0) &&(pointer.offset == 0){
                break;
            }
            pointers.push(pointer);
        }
        for i in 0.. pointers.len() {
            let offset = pointers[i].offset;
            reader.seek(SeekFrom::Start(offset as u64 + REFERENCE )).expect("Seek error!!");
            let  (code,start_3c,end_3c) = decode(&mut reader);
            pointers[i].codes =  code;
            pointers[i].start_3c =  start_3c;
            pointers[i].end_3c =  end_3c;
        }
        let script_ys_v = ScriptYsV{
            contents: pointers
        };
        script_ys_v


    }

    pub fn to_bin(&self,out_name: &PathBuf){
        let mut wr = File::create(out_name).expect("Buat *.bin gagal");
        let mut buffw:Vec<u8> = Vec::new();
        let mut b_wr = Cursor::new(&mut buffw);
      //  bytes_script_writer.re
        let mut n_off:u32;
        for content in  &self.contents  {
            n_off = b_wr.stream_pos().unwrap() as u32;
            wr.write(&content.idx.to_le_bytes()).unwrap();
            wr.write(&n_off.to_le_bytes()).unwrap();
            let mut start_3c:Vec<u64> = Vec::new();
            let mut end_3c:Vec<u64> = Vec::new();
            let mut n:u32 = 0;
            for  c in &content.codes {
                for x in content.start_3c.clone() {
                    if x == n{
                        start_3c.push(b_wr.stream_pos().unwrap());

                    }
                }
                for x in content.end_3c.clone() {
                    if x == n{
                        end_3c.push(b_wr.stream_pos().unwrap());
                    }
                }
                match c {

                    Code::C00{}=>{
                        b_wr.write_v(0x00u8);
                    }
                    Code::C01{a0}=>{
                        b_wr.write_v(0x01u8);
                        b_wr.write_v(a0);
                    }
                    Code::C02{a0}=>{
                        b_wr.write_v(0x02u8);
                        b_wr.write_v(a0);}
                    Code::C03{a0}=>{
                        b_wr.write_v(0x03u8);
                        b_wr.write_v(a0);}
                    Code::C04{}=>{
                        b_wr.write_v(0x04u8);
                    }
                    Code::C05{}=>{
                        b_wr.write_v(0x05u8);
                    }
                    Code::C06{}=>{
                        b_wr.write_v(0x06u8);
                    }
                    Code::C07{a0}=>{
                        b_wr.write_v(0x07u8);
                        b_wr.write_v(a0);
                    }
                    Code::C08{ xstr: str }=>{
                        b_wr.write_v(0x08u8);
                        let (bytes_string,_,_) = SHIFT_JIS.encode(&str);
                        b_wr.write(&bytes_string).unwrap();
                        b_wr.write_v(0x00u8);
                    }
                    Code::C09{ xstr: str }=>{
                        b_wr.write_v(0x09u8);
                        let (bytes_string,_,_) = SHIFT_JIS.encode(&str);
                        b_wr.write(&bytes_string).unwrap();
                        b_wr.write_v(0x00u8);
                    }
                    Code::C0A{a0,a1,mut len,str}=>{
                        b_wr.write_v(0x0Au8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);
                        let bytes_string = get_bytes_string(str);
                        len = (bytes_string.len() as u16) | 0x8000;
                        b_wr.write_v(len);
                        b_wr.write(&bytes_string).unwrap();
                    }
                    Code::C0B{}=>{ b_wr.write_v(0x0Bu8);}
                    Code::C0C{}=>{ b_wr.write_v(0x0Cu8);}
                    Code::C0D{}=>{ b_wr.write_v(0x0Du8);}
                    Code::C0E{a0}=>{
                        b_wr.write_v(0x0Eu8);
                        b_wr.write_v(a0);}
                    Code::C0F{}=>{ b_wr.write_v(0x0Fu8);}
                    Code::C10{}=>{ b_wr.write_v(0x10u8);}
                    Code::C11{}=>{ b_wr.write_v(0x11u8);}
                    Code::C12{}=>{ b_wr.write_v(0x12u8);}
                    Code::C13{a0,a1}=>{
                        b_wr.write_v(0x13u8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);}
                    Code::C14{a0}=>{
                        b_wr.write_v(0x14u8);
                        b_wr.write_v(a0);}
                    Code::C15{a0}=>{
                        b_wr.write_v(0x15u8);
                        b_wr.write_v(a0);}
                    Code::C16{a0,a1,a2}=>{
                        b_wr.write_v(0x16u8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);
                        b_wr.write_v(a2);}
                    Code::C17{str,a0}=>{
                        b_wr.write_v(0x17u8);
                        let bytes_string = get_bytes_string(str);
                        b_wr.write(&bytes_string).unwrap();
                        b_wr.write_v(0x00u8);
                        b_wr.write_v(a0);}
                    Code::C18{a0,a1,a2}=>{
                        b_wr.write_v(0x18u8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);
                        b_wr.write_v(a2);}
                    Code::C19{a0,a1,a2,a3}=>{
                        b_wr.write_v(0x19u8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);
                        b_wr.write_v(a2);
                        b_wr.write_v(a3);}
                    Code::C1A{a0,a1}=>{
                        b_wr.write_v(0x1Au8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);}
                    Code::C1B{a0,a1}=>{
                        b_wr.write_v(0x1Bu8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);}
                    Code::C1C{a0,a1}=>{
                        b_wr.write_v(0x1Cu8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);}
                    Code::C1D{a0,a1}=>{
                        b_wr.write_v(0x1Du8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);}
                    Code::C1E{a0,a1,a2,a3,a4}=>{
                        b_wr.write_v(0x1Eu8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);
                        b_wr.write_v(a2);
                        b_wr.write_v(a3);
                        b_wr.write_v(a4);}
                    Code::C1F{a0,a1,a2}=>{
                        b_wr.write_v(0x1Fu8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);
                        b_wr.write_v(a2);
                    }
                    Code::C20{a0,a1}=>{
                        b_wr.write_v(0x20u8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);}
                    Code::C21{a0,a1}=>{
                        b_wr.write_v(0x21u8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);}
                    Code::C22{a0,a1}=>{
                        b_wr.write_v(0x22u8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);}
                    Code::C23{a0,a1}=>{
                        b_wr.write_v(0x23u8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);}
                    Code::C24{a0,a1,mut len,ask, choices}=>{
                        b_wr.write_v(0x24u8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);
                        let bytes_string = get_bytes_string(&ask);
                        len = (bytes_string.len() as u16) | 0x8000;
                        b_wr.write_v(len);
                        b_wr.write(&bytes_string).unwrap();
                        for  choice in choices {
                            let bytes_string = get_bytes_string(&choice.choice);
                            len = (bytes_string.len() as u16) | 0x8000;
                            b_wr.write_v(len);
                            b_wr.write(&bytes_string).unwrap();
                            b_wr.write_v(choice.end_0);
                            b_wr.write_v(choice.end_1);
                        }
                    }
                    Code::C25{a0,mut len,ask, choices}=>{
                        b_wr.write_v(0x25u8);
                        b_wr.write_v(a0);
                        let bytes_string = get_bytes_string(&ask);
                        len = (bytes_string.len() as u16) | 0x8000;
                        b_wr.write_v(len);
                        b_wr.write(&bytes_string).unwrap();
                        for  choice in choices {
                            let bytes_string = get_bytes_string(&choice.choice);
                            len = (bytes_string.len() as u16) | 0x8000;
                            b_wr.write_v(len);
                            b_wr.write(&bytes_string).unwrap();
                            b_wr.write_v(choice.end_0);
                            b_wr.write_v(choice.end_1);
                        }
                    }
                    Code::C26{a0,mut len,ask, choices}=>{
                        b_wr.write_v(0x26u8);
                        b_wr.write_v(a0);
                        let bytes_string = get_bytes_string(&ask);
                        len = (bytes_string.len() as u16) | 0x8000;
                        b_wr.write_v(len);
                        b_wr.write(&bytes_string).unwrap();
                        for choice in choices {
                            let bytes_string = get_bytes_string(&choice.choice);
                            len = (bytes_string.len() as u16) | 0x8000;
                            b_wr.write_v(len);
                            b_wr.write(&bytes_string).unwrap();
                            b_wr.write_v(choice.end_0);
                            b_wr.write_v(choice.end_1);
                        }
                    }
                    Code::C27{a0,a1,mut len,ask, choices}=>{
                        b_wr.write_v(0x27u8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);
                        let bytes_string = get_bytes_string(&ask);
                        len = (bytes_string.len() as u16) | 0x8000;
                        b_wr.write_v(len);
                        b_wr.write(&bytes_string).unwrap();
                        for choice in choices {
                            let bytes_string = get_bytes_string(&choice.choice);
                            len = (bytes_string.len() as u16) | 0x8000;
                            b_wr.write_v(len);
                            b_wr.write(&bytes_string).unwrap();
                            b_wr.write_v(choice.end_0);
                            b_wr.write_v(choice.end_1);
                        }
                    }
                    Code::C28{a0,mut len,ask, choices}=>{
                        b_wr.write_v(0x28u8);
                        b_wr.write_v(a0);
                        let bytes_string = get_bytes_string(&ask);
                        len = (bytes_string.len() as u16) | 0x8000;
                        b_wr.write_v(len);
                        b_wr.write(&bytes_string).unwrap();
                        for choice in choices {
                            let bytes_string = get_bytes_string(&choice.choice);
                            len = (bytes_string.len() as u16) | 0x8000;
                            b_wr.write_v(len);
                            b_wr.write(&bytes_string).unwrap();
                            b_wr.write_v(choice.end_0);
                            b_wr.write_v(choice.end_1);
                        }
                    }
                    Code::C29{a0,mut len,ask, choices}=>{
                        b_wr.write_v(0x29u8);
                        b_wr.write_v(a0);
                        let bytes_string = get_bytes_string(&ask);
                        len = (bytes_string.len() as u16) | 0x8000;
                        b_wr.write_v(len);
                        b_wr.write(&bytes_string).unwrap();
                        for choice in choices {
                            let bytes_string = get_bytes_string(&choice.choice);
                            len = (bytes_string.len() as u16) | 0x8000;
                            b_wr.write_v(len);
                            b_wr.write(&bytes_string).unwrap();
                            b_wr.write_v(choice.end_0);
                            b_wr.write_v(choice.end_1);
                        }
                    }
                    Code::C2A{a0,a1}=>{
                        b_wr.write_v(0x2Au8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);}
                    Code::C2B{a0}=>{
                        b_wr.write_v(0x2Bu8);
                        b_wr.write_v(a0);}
                    Code::C2C{a0}=>{
                        b_wr.write_v(0x2Cu8);
                        b_wr.write_v(a0);}
                    Code::C2D{a0,a1,a2,a3,a4}=>{
                        b_wr.write_v(0x2Du8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);
                        b_wr.write_v(a2);
                        b_wr.write_v(a3);
                        b_wr.write_v(a4);}
                    Code::C2E{ xstr: str }=>{
                        b_wr.write_v(0x2Eu8);
                        let bytes_string = get_bytes_string(&str);
                        b_wr.write(&bytes_string).unwrap();
                        b_wr.write_v(0x00u8);
                    }
                    Code::C2F{a0}=>{
                        b_wr.write_v(0x2Fu8);
                        b_wr.write_v(a0);}
                    Code::C30{a0}=>{
                        b_wr.write_v(0x30u8);
                        b_wr.write_v(a0);}
                    Code::C31{a0}=>{
                        b_wr.write_v(0x31u8);
                        b_wr.write_v(a0);}
                    Code::C32{a0}=>{
                        b_wr.write_v(0x32u8);
                        b_wr.write_v(a0);}
                    Code::C33{a0}=>{
                        b_wr.write_v(0x33u8);
                        b_wr.write_v(a0);}
                    Code::C34{}=>{ b_wr.write_v(0x34u8);}
                    Code::C35{a0,a1,mut len,str,}=>{
                        b_wr.write_v(0x35u8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);
                        let bytes_string = get_bytes_string(&str);
                        len = (bytes_string.len() as u16) | 0x8000;
                        b_wr.write_v(len);
                        b_wr.write(&bytes_string).unwrap();
                    }
                    Code::C36{a0,a1,a2,a3,a4}=>{
                        b_wr.write_v(0x36u8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);
                        b_wr.write_v(a2);
                        b_wr.write_v(a3);
                        b_wr.write_v(a4);}
                    Code::C37{a0,a1,a2,a3,a4}=>{
                        b_wr.write_v(0x37u8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);
                        b_wr.write_v(a2);
                        b_wr.write_v(a3);
                        b_wr.write_v(a4);}
                    Code::C38{a0,a1,a2}=>{
                        b_wr.write_v(0x38u8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);
                        b_wr.write_v(a2);}
                    Code::C39{a0,a1,a2}=>{
                        b_wr.write_v(0x39u8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);
                        b_wr.write_v(a2);}
                    Code::C3A{a0,a1}=>{
                        b_wr.write_v(0x3Au8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);}
                    Code::C3B{a0,mut len,str}=>{
                        b_wr.write_v(0x3Bu8);
                        b_wr.write_v(a0);
                        let bytes_string = get_bytes_string(&str);
                        len = (bytes_string.len() as u16) | 0x8000;
                        b_wr.write_v(len);
                        b_wr.write(&bytes_string).unwrap();
                    }
                    Code::C3C{ jump: a0,a1}=>{
                        b_wr.write_v(0x3Cu8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);}
                    Code::C3D{a0}=>{
                        b_wr.write_v(0x3Du8);
                        b_wr.write_v(a0);}
                    Code::C3E{a0}=>{
                        b_wr.write_v(0x3Eu8);
                        b_wr.write_v(a0);}
                    Code::C3F{a0}=>{
                        b_wr.write_v(0x3Fu8);
                        b_wr.write_v(a0);}
                    Code::C40{a0}=>{
                        b_wr.write_v(0x40u8);
                        b_wr.write_v(a0);}
                    Code::C41{a0}=>{
                        b_wr.write_v(0x41u8);
                        b_wr.write_v(a0);}
                    Code::C42{a0}=>{
                        b_wr.write_v(0x42u8);
                        b_wr.write_v(a0);}
                    Code::C43{a0}=>{
                        b_wr.write_v(0x43u8);
                        b_wr.write_v(a0);}
                    Code::C44{a0,a1}=>{
                        b_wr.write_v(0x44u8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);}
                    Code::C45{a0}=>{
                        b_wr.write_v(0x45u8);
                        b_wr.write_v(a0);}
                    Code::C46{}=>{ b_wr.write_v(0x46u8);}
                    Code::C47{a0}=>{
                        b_wr.write_v(0x47u8);
                        b_wr.write_v(a0);}
                    Code::C48{a0}=>{
                        b_wr.write_v(0x48u8);
                        b_wr.write_v(a0);}
                    Code::C49{a0}=>{
                        b_wr.write_v(0x49u8);
                        b_wr.write_v(a0);}
                    Code::C4A{a0,a1}=>{
                        b_wr.write_v(0x4Au8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);}
                    Code::C4B{a0,a1}=>{
                        b_wr.write_v(0x4Bu8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);}
                    Code::C4C{a0}=>{
                        b_wr.write_v(0x4Cu8);
                        b_wr.write_v(a0);}
                    Code::C4D{a0}=>{
                        b_wr.write_v(0x4Du8);
                        b_wr.write_v(a0);}
                    Code::C4E{}=>{ b_wr.write_v(0x4Eu8);}
                    Code::C4F{a0}=>{
                        b_wr.write_v(0x4Fu8);
                        b_wr.write_v(a0);}
                    Code::C50{a0,a1}=>{
                        b_wr.write_v(0x50u8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);}
                    Code::C51{}=>{ b_wr.write_v(0x51u8);}
                    Code::C52{a0,mut len,str}=>{
                        b_wr.write_v(0x52u8);
                        b_wr.write_v(a0);
                        let bytes_string = get_bytes_string(&str);
                        len = (bytes_string.len() as u16) | 0x8000;
                        b_wr.write_v(len);
                        b_wr.write(&bytes_string).unwrap();
                    }
                    Code::C53{a0}=>{
                        b_wr.write_v(0x53u8);
                        b_wr.write_v(a0);}
                    Code::C54{ xstr: str }=>{
                        b_wr.write_v(0x54u8);
                        let bytes_string = get_bytes_string(&str);
                        b_wr.write(&bytes_string).unwrap();
                        b_wr.write_v(0x00u8);

                    }
                    Code::C55{}=>{ b_wr.write_v(0x55u8);}
                    Code::C56{a0}=>{
                        b_wr.write_v(0x56u8);
                        b_wr.write_v(a0);}
                    Code::C57{a0}=>{
                        b_wr.write_v(0x57u8);
                        b_wr.write_v(a0);}
                    Code::C58{a0}=>{
                        b_wr.write_v(0x58u8);
                        b_wr.write_v(a0);}
                    Code::C59{a0}=>{
                        b_wr.write_v(0x59u8);
                        b_wr.write_v(a0);}
                    Code::C5A{}=>{ b_wr.write_v(0x5Au8);}
                    Code::C5B{a0,a1}=>{
                        b_wr.write_v(0x5Bu8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);}
                    Code::C5C{a0,a1}=>{
                        b_wr.write_v(0x5Cu8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);}
                    Code::C5D{a0,a1}=>{
                        b_wr.write_v(0x5Du8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);}
                    Code::C5E{a0}=>{
                        b_wr.write_v(0x5Eu8);
                        b_wr.write_v(a0);}
                    Code::C5F{a0}=>{
                        b_wr.write_v(0x5Fu8);
                        b_wr.write_v(a0);}
                    Code::C60{a0}=>{
                        b_wr.write_v(0x60u8);
                        b_wr.write_v(a0);}
                    Code::C61{a0}=>{
                        b_wr.write_v(0x61u8);
                        b_wr.write_v(a0);}
                    Code::C62{a0,a1}=>{
                        b_wr.write_v(0x62u8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);}
                    Code::C63{a0,a1}=>{
                        b_wr.write_v(0x63u8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);}
                    Code::C64{a0,a1,a2,a3,a4,a5,a6}=>{
                        b_wr.write_v(0x64u8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);
                        b_wr.write_v(a2);
                        b_wr.write_v(a3);
                        b_wr.write_v(a4);
                        b_wr.write_v(a5);
                        b_wr.write_v(a6);}
                    Code::C65{a0}=>{
                        b_wr.write_v(0x65u8);
                        b_wr.write_v(a0);}
                    Code::C66{a0,a1,a2}=>{
                        b_wr.write_v(0x66u8);
                        b_wr.write_v(a0);
                        b_wr.write_v(a1);
                        b_wr.write_v(a2);}
                    Code::C67{}=>{ b_wr.write_v(0x67u8);}
                    Code::End{}=>{ b_wr.write_v(0xFFu8);}

                }
                n+=1;

            }
            let tmp =  b_wr.stream_pos().unwrap();
            if start_3c.len()==end_3c.len(){
                for n in 0..start_3c.len(){
                    b_wr.seek(SeekFrom::Start(start_3c[n]+1)).unwrap();
                    let new_3c_len = (end_3c[n]-start_3c[n]-1) as u16;
                    b_wr.write(&new_3c_len.to_le_bytes()).unwrap();
                }
            }
            b_wr.seek(SeekFrom::Start(tmp)).unwrap();
        }

        wr.write_padding(REFERENCE, 0);
        b_wr.seek(SeekFrom::Start(0)).unwrap();
        let mut bytes_script = Vec::new();
        b_wr.read_to_end(&mut bytes_script).unwrap();
        wr.write(&bytes_script).unwrap();
        wr.write_padding(0x800, 0);
    }

}
