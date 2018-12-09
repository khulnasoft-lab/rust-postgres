use linked_hash_map::LinkedHashMap;
use phf_codegen;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

const ERRCODES_TXT: &'static str = include_str!("errcodes.txt");

pub fn build(path: &Path) {
    let mut file = BufWriter::new(File::create(path.join("error/sqlstate.rs")).unwrap());

    let codes = parse_codes();

    make_type(&mut file);
    make_consts(&codes, &mut file);
    make_map(&codes, &mut file);
}

fn parse_codes() -> LinkedHashMap<String, Vec<String>> {
    let mut codes = LinkedHashMap::new();

    for line in ERRCODES_TXT.lines() {
        if line.starts_with("#") || line.starts_with("Section") || line.trim().is_empty() {
            continue;
        }

        let mut it = line.split_whitespace();
        let code = it.next().unwrap().to_owned();
        it.next();
        let name = it.next().unwrap().replace("ERRCODE_", "");

        codes.entry(code).or_insert_with(Vec::new).push(name);
    }

    codes
}

fn make_type(file: &mut BufWriter<File>) {
    write!(
        file,
        "// Autogenerated file - DO NOT EDIT
use phf;
use std::borrow::Cow;

/// A SQLSTATE error code
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct SqlState(Cow<'static, str>);

impl SqlState {{
    /// Creates a `SqlState` from its error code.
    pub fn from_code(s: &str) -> SqlState {{
        match SQLSTATE_MAP.get(s) {{
            Some(state) => state.clone(),
            None => SqlState(Cow::Owned(s.to_string())),
        }}
    }}

    /// Returns the error code corresponding to the `SqlState`.
    pub fn code(&self) -> &str {{
        &self.0
    }}
"
    )
    .unwrap();
}

fn make_consts(codes: &LinkedHashMap<String, Vec<String>>, file: &mut BufWriter<File>) {
    for (code, names) in codes {
        for name in names {
            write!(
                file,
                r#"
    /// {code}
    pub const {name}: SqlState = SqlState(Cow::Borrowed("{code}"));
"#,
                name = name,
                code = code,
            )
            .unwrap();
        }
    }

    write!(file, "}}").unwrap();
}

fn make_map(codes: &LinkedHashMap<String, Vec<String>>, file: &mut BufWriter<File>) {
    write!(
        file,
        "
#[cfg_attr(rustfmt, rustfmt_skip)]
static SQLSTATE_MAP: phf::Map<&'static str, SqlState> = "
    )
    .unwrap();
    let mut builder = phf_codegen::Map::new();
    for (code, names) in codes {
        builder.entry(&**code, &format!("SqlState::{}", &names[0]));
    }
    builder.build(file).unwrap();
    write!(file, ";\n").unwrap();
}
