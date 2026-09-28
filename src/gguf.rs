//! Just enough of the GGUF header to read a model's attention head sizes, so llama-server
//! isn't asked for a KV cache type the model can't use. Pure, so it can be unit-tested.

use std::collections::HashMap;
use std::io::{self, Read};

const MAGIC: &[u8; 4] = b"GGUF";
/// Nothing in a real header comes close; anything bigger means a corrupt or foreign file.
const MAX_LEN: u64 = 1 << 30;

/// (key, value) head sizes from the metadata: `<arch>.attention.key_length` /
/// `value_length` when present, otherwise `embedding_length / head_count` (llama.cpp's own
/// defaults). None if the file isn't GGUF v2+ or doesn't say.
pub fn head_dims(mut r: impl Read) -> Option<(u32, u32)> {
    let mut magic = [0; 4];
    r.read_exact(&mut magic).ok()?;
    if &magic != MAGIC || !(2..=3).contains(&read_u32(&mut r).ok()?) {
        return None;
    }
    let _tensors = read_u64(&mut r).ok()?;
    let kvs = read_u64(&mut r).ok()?;
    let mut arch = None;
    let mut ints: HashMap<String, u64> = HashMap::new();
    for _ in 0..kvs {
        let key = read_string(&mut r).ok()?;
        let ty = read_u32(&mut r).ok()?;
        if key == "general.architecture" && ty == STRING {
            arch = Some(read_string(&mut r).ok()?);
        } else if let Some(v) = read_int(&mut r, ty).ok()? {
            ints.insert(key, v);
        } else {
            skip_value(&mut r, ty).ok()?;
        }
        if let Some(a) = &arch {
            let has = |k: &str| ints.contains_key(&format!("{a}.attention.{k}"));
            if has("key_length") && has("value_length") {
                break;
            }
        }
    }
    let a = arch?;
    let get = |k: &str| ints.get(&format!("{a}.{k}")).copied();
    let k = match get("attention.key_length") {
        Some(k) => k,
        None => get("embedding_length")?.checked_div(get("attention.head_count")?)?,
    };
    let v = get("attention.value_length").unwrap_or(k);
    Some((u32::try_from(k).ok()?, u32::try_from(v).ok()?))
}

const STRING: u32 = 8;
const ARRAY: u32 = 9;

/// Size of a fixed-size value type, None for strings, arrays and unknown types.
fn scalar_size(ty: u32) -> Option<u64> {
    match ty {
        0 | 1 | 7 => Some(1), // u8, i8, bool
        2 | 3 => Some(2),     // u16, i16
        4..=6 => Some(4),     // u32, i32, f32
        10..=12 => Some(8),   // u64, i64, f64
        _ => None,
    }
}

/// An unsigned-integer-valued scalar, read; any other type is left unread (Ok(None)).
fn read_int(r: &mut impl Read, ty: u32) -> io::Result<Option<u64>> {
    Ok(match ty {
        0 => Some(read_n::<1>(r)?[0] as u64),
        2 => Some(u16::from_le_bytes(read_n(r)?) as u64),
        4 => Some(read_u32(r)? as u64),
        5 => u64::try_from(i32::from_le_bytes(read_n(r)?)).ok(),
        10 => Some(read_u64(r)?),
        _ => None,
    })
}

fn skip_value(r: &mut impl Read, ty: u32) -> io::Result<()> {
    match ty {
        STRING => {
            let len = checked_len(read_u64(r)?)?;
            skip(r, len)
        }
        ARRAY => {
            let elem = read_u32(r)?;
            let n = checked_len(read_u64(r)?)?;
            if let Some(size) = scalar_size(elem) {
                skip(r, n * size)
            } else if elem == STRING {
                (0..n).try_for_each(|_| skip_value(r, STRING))
            } else {
                Err(bad("unsupported array element type"))
            }
        }
        t => skip(r, scalar_size(t).ok_or_else(|| bad("unknown value type"))?),
    }
}

fn skip(r: &mut impl Read, n: u64) -> io::Result<()> {
    if io::copy(&mut r.take(n), &mut io::sink())? != n {
        return Err(io::ErrorKind::UnexpectedEof.into());
    }
    Ok(())
}

fn read_string(r: &mut impl Read) -> io::Result<String> {
    let len = checked_len(read_u64(r)?)?;
    let mut buf = Vec::with_capacity(len.min(4096) as usize);
    if r.take(len).read_to_end(&mut buf)? as u64 != len {
        return Err(io::ErrorKind::UnexpectedEof.into());
    }
    String::from_utf8(buf).map_err(|_| bad("key isn't UTF-8"))
}

fn checked_len(n: u64) -> io::Result<u64> {
    if n > MAX_LEN {
        return Err(bad("length out of range"));
    }
    Ok(n)
}

fn read_n<const N: usize>(r: &mut impl Read) -> io::Result<[u8; N]> {
    let mut b = [0; N];
    r.read_exact(&mut b)?;
    Ok(b)
}

fn read_u32(r: &mut impl Read) -> io::Result<u32> {
    Ok(u32::from_le_bytes(read_n(r)?))
}

fn read_u64(r: &mut impl Read) -> io::Result<u64> {
    Ok(u64::from_le_bytes(read_n(r)?))
}

fn bad(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    enum V<'a> {
        U32(u32),
        I32(i32),
        U64(u64),
        F32(f32),
        Str(&'a str),
        Strs(&'a [&'a str]),
        I32s(&'a [i32]),
    }

    fn string(out: &mut Vec<u8>, s: &str) {
        out.extend((s.len() as u64).to_le_bytes());
        out.extend(s.as_bytes());
    }

    fn gguf(version: u32, kvs: &[(&str, V)]) -> Vec<u8> {
        let mut out = b"GGUF".to_vec();
        out.extend(version.to_le_bytes());
        out.extend(0u64.to_le_bytes());
        out.extend((kvs.len() as u64).to_le_bytes());
        for (k, v) in kvs {
            string(&mut out, k);
            match v {
                V::U32(x) => out.extend(4u32.to_le_bytes().into_iter().chain(x.to_le_bytes())),
                V::I32(x) => out.extend(5u32.to_le_bytes().into_iter().chain(x.to_le_bytes())),
                V::U64(x) => out.extend(10u32.to_le_bytes().into_iter().chain(x.to_le_bytes())),
                V::F32(x) => out.extend(6u32.to_le_bytes().into_iter().chain(x.to_le_bytes())),
                V::Str(s) => {
                    out.extend(8u32.to_le_bytes());
                    string(&mut out, s);
                }
                V::Strs(ss) => {
                    out.extend(9u32.to_le_bytes());
                    out.extend(8u32.to_le_bytes());
                    out.extend((ss.len() as u64).to_le_bytes());
                    ss.iter().for_each(|s| string(&mut out, s));
                }
                V::I32s(xs) => {
                    out.extend(9u32.to_le_bytes());
                    out.extend(5u32.to_le_bytes());
                    out.extend((xs.len() as u64).to_le_bytes());
                    xs.iter().for_each(|x| out.extend(x.to_le_bytes()));
                }
            }
        }
        out
    }

    #[test]
    fn head_size_from_embedding_and_heads() {
        // stories260K: 64 / 8 = 8, too small for q8_0's 32-element blocks
        let f = gguf(
            3,
            &[
                ("general.architecture", V::Str("llama")),
                ("general.name", V::Str("stories260K")),
                ("tokenizer.ggml.tokens", V::Strs(&["<unk>", "<s>", "</s>"])),
                ("tokenizer.ggml.token_type", V::I32s(&[2, 3, 3])),
                ("llama.rope.freq_base", V::F32(10000.0)),
                ("llama.embedding_length", V::U32(64)),
                ("llama.attention.head_count", V::U32(8)),
            ],
        );
        assert_eq!(head_dims(&f[..]), Some((8, 8)));
    }

    #[test]
    fn explicit_key_and_value_lengths_win() {
        let f = gguf(
            3,
            &[
                ("general.architecture", V::Str("qwen3")),
                ("qwen3.embedding_length", V::U64(5120)),
                ("qwen3.attention.head_count", V::I32(40)),
                ("qwen3.attention.key_length", V::U32(192)),
                ("qwen3.attention.value_length", V::U32(128)),
            ],
        );
        assert_eq!(head_dims(&f[..]), Some((192, 128)));
        // value_length defaults to key_length
        let f = gguf(
            2,
            &[
                ("general.architecture", V::Str("gemma")),
                ("gemma.attention.key_length", V::U32(256)),
            ],
        );
        assert_eq!(head_dims(&f[..]), Some((256, 256)));
    }

    #[test]
    fn unknown_or_malformed_is_none() {
        assert_eq!(head_dims(&b"GGML...."[..]), None);
        assert_eq!(head_dims(&gguf(1, &[])[..]), None);
        assert_eq!(head_dims(&gguf(3, &[])[..]), None);
        // no architecture
        let f = gguf(3, &[("llama.attention.key_length", V::U32(128))]);
        assert_eq!(head_dims(&f[..]), None);
        // zero heads
        let f = gguf(
            3,
            &[
                ("general.architecture", V::Str("llama")),
                ("llama.embedding_length", V::U32(64)),
                ("llama.attention.head_count", V::U32(0)),
            ],
        );
        assert_eq!(head_dims(&f[..]), None);
        // truncated mid-header
        let f = gguf(
            3,
            &[
                ("general.architecture", V::Str("llama")),
                ("llama.attention.key_length", V::U32(128)),
            ],
        );
        assert_eq!(head_dims(&f[..f.len() - 2]), None);
        // absurd string length
        let mut f = gguf(3, &[]);
        f[16..24].copy_from_slice(&1u64.to_le_bytes());
        f.extend(u64::MAX.to_le_bytes());
        assert_eq!(head_dims(&f[..]), None);
    }
}
