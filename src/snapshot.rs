use nom::{IResult, be_u8, be_u16, le_u16};

#[derive(Debug, Clone, Copy)]
pub struct Z80Header {
    pub a: u8,
    pub f: u8,
    pub bc: u16,
    pub hl: u16,
    pub pc: u16,
    pub sp: u16,
    pub ir: u8,
    pub r: u8,
    pub misc1: u8,
    pub de: u16,
    pub bc_alt: u16,
    pub de_alt: u16,
    pub hl_alt: u16,
    pub af_alt: u16,
    pub iy: u16,
    pub ix: u16,
    pub iff1: bool,
    pub iff2: bool,
    pub misc2: u8,
}

named!(
    compressed_data<&[u8], Vec<u8>>,
    do_parse!(
      tag!( &[0xED, 0xED][..] ) >>
      xx: be_u8 >>
      yy: be_u8 >>
      ({
          let v:Vec<u8> = vec![yy; xx as usize];
          v
      })
    )
);

pub fn header(input:&[u8]) -> IResult<&[u8], (Z80Header, Vec<u8>)> {
    do_parse!(input,
      a: be_u8 >>
      f: be_u8 >>
      bc: le_u16 >>
      hl: le_u16 >>
      pc: le_u16 >>
      sp: le_u16 >>
      ir: be_u8 >>
      r: be_u8 >>
      misc1: be_u8 >>
      de: le_u16 >>
      bc_alt: le_u16 >>
      de_alt: le_u16 >>
      hl_alt: le_u16 >>
      af_alt: be_u16 >>
      iy: le_u16 >>
      ix: le_u16 >>
      iff1: be_u8 >>
      iff2: be_u8 >>
      misc2: be_u8 >>
      data: fold_many1!(
          alt_complete!(
            compressed_data |
            map!(tag!( &[0x00, 0xED, 0xED ,0x00][..] ), |x| x.to_vec()) |
            map!(take!(1), |x| x.to_vec())
          ),
          Vec::new(),
          |mut acc: Vec<u8>, item: Vec<u8> | {
              acc.extend(item);
              acc
          }) >>
     (
       (
        Z80Header {
          a,
          f,
          bc,
          hl,
          pc,
          sp,
          ir,
          r: r & 0b01111111,
          misc1,
          de,
          bc_alt,
          de_alt,
          hl_alt,
          af_alt,
          iy,
          ix,
          iff1: iff1 != 0,
          iff2: iff2 != 0,
          misc2
        },
        data
       )
     )
    )
}

pub fn parse(input:&[u8]) -> Option<(Z80Header, Vec<u8>)> {
    if let IResult::Done(_, snapshot) = header(input) {
        Some(snapshot)
    } else {
        None
    }
}

