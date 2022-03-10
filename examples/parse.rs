use std::env::args;
use std::fs;
use vmdl::mdl::Mdl;

fn main() -> Result<(), vmdl::ModelError> {
    let mut args = args();
    let _ = args.next();
    let data = fs::read(args.next().expect("No demo file provided"))?;
    let mdl = Mdl::read(&data)?;

    dbg!(mdl.header);

    Ok(())
}
