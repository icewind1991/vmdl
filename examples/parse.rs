fn main() -> Result<(), vmdl::MdlError> {
    let mut args = std::env::args();
    let _ = args.next();
    let data = std::fs::read(args.next().expect("No demo file provided"))?;
    let mdl = vmdl::Mdl::read(&data)?;

    dbg!(mdl.bones);

    Ok(())
}
