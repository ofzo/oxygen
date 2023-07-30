pub fn leb_encode_len(buf: &Vec<u8>) -> u32 {
    let mut count = 0;
    let len = buf.len();
    while count < len && buf[count] >= 0b1000_0000 {
        count += 1;
    }
    count += 1;
    return count as u32;
}

/// LEB128（Little Endian Base 128） 变长编码格式目的是节约空间
/// 对于 32 位整数，编码后可能是 1 到 5 个字节
/// 对于 64 位整数，编码后可能是 1 到 10 个字节
/// 越小的整数，编码后占用的字节数就越小
///
/// https://en.wikipedia.org/wiki/LEB128#Decode_unsigned_integer
///
/// 针对无符号整数的 LEB128 编码特点：
/// 1. 采用小端编码方式，即低位字节在前，高位字节在后
/// 2. 采用 128 进制，每 7 个比特为一组，由一个字节的后 7 位承载，空出来的最高位是标记位，1 表示后面还有后续字节，0 表示没有
/// 例如：LEB128 编码为 11100101 10001110 00100110，解码为 000 0100110 0001110 1100101
/// 1100101 0001110 0100110
/// 000 0100110 0001110 1100101
/// 0000_1001 1000_0111 0110_0101
/// 0110_0101 1000_0111 0000_1001
/// 01100101 10000111 00001001
/// 624485
/// 注：0x80 -- 10000000    0x7f -- 01111111
///
/// 针对有符号整数的 LEB128 编码，与上面无符号的完全相同，
/// 只有最后一个字节的第二高位是符号位，如果是 1，表示这是一个负数，需将高位全部补全为 1，如果是 0，表示这是一个正数，需将高位全部补全为 0
pub fn decode_leb_i32(buf: &Vec<u8>) -> (i32, usize) {
    let length = leb_encode_len(buf) as usize;

    let buf = buf[0..length].to_vec();

    if buf.last().unwrap() & 0b0100_0000 > 0 {
        let mut r = -1i32;
        for i in (0..length).rev() {
            let byte = if i == length - 1 {
                r = r << 6;
                (buf[i] & 0b0011_1111) | 0b1100_0000
            } else {
                r = r << 7;
                buf[i] & 0b0111_1111
            } as i32;

            r |= byte;
        }
        (r, length)
    } else {
        let mut r = 0i32;
        let mut shift = 0;
        for i in 0..length {
            let byte = (buf[i] & 0b0111_1111) as i32;

            let byte = byte << shift;
            shift += 7;

            r |= byte;
        }
        (r, length)
    }
}

pub fn decode_leb_i64(buf: &Vec<u8>) -> (i64, usize) {
    let length = leb_encode_len(buf) as usize;

    let buf = buf[0..length].to_vec();

    if buf.last().unwrap() & 0b0100_0000 > 0 {
        let mut r = -1i64;
        for i in (0..length).rev() {
            let byte = if i == length - 1 {
                r = r << 6;
                (buf[i] & 0b0011_1111) | 0b1100_0000
            } else {
                r = r << 7;
                buf[i] & 0b0111_1111
            } as i64;

            r |= byte;
        }
        (r, length)
    } else {
        let mut r = 0i64;
        let mut shift = 0;
        for i in 0..length {
            let byte = (buf[i] & 0b0111_1111) as i64;

            let byte = byte << shift;
            shift += 7;

            r |= byte;
        }
        (r, length)
    }
}

pub fn decode_leb_u32(buf: &Vec<u8>) -> (u32, usize) {
    let length = leb_encode_len(buf) as usize; // length = 1

    let buf = buf[0..length].to_vec();
    let mut r = 0u32;
    let mut shift = 0;
    for i in 0..length {
        let byte = (buf[i] & 0b0111_1111) as u32;

        let byte = byte << shift;
        shift += 7;

        r |= byte;
    }
    (r, length)
}

pub fn decode_leb_u64(buf: &Vec<u8>) -> (u64, usize) {
    let length = leb_encode_len(buf) as usize; // length = 1

    let buf = buf[0..length].to_vec();
    let mut r = 0u64;
    let mut shift = 0;
    for i in 0..length {
        let byte = (buf[i] & 0b0111_1111) as u64;

        let byte = byte << shift;
        shift += 7;

        r |= byte;
    }
    (r, length)
}

#[test]
fn test_bit_write() {
    let mut buffer: Vec<u8> = vec![0x8c, 0x80, 0x80, 0x80, 0x00];

    let buf = decode_leb_u32(&mut buffer);

    assert_eq!(buf, (12, 5));
}
#[test]
fn test_decode_leb_u32() {
    let mut buffer: Vec<u8> = vec![0xf0, 0xff, 0xff, 0xff, 0x0f, 0xff, 0xff, 0x7f];
    let r = decode_leb_u32(&mut buffer);
    println!(" r = {}", r.0);
}
