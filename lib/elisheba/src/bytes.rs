use tokio::io::{
    AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter, ReadHalf, WriteHalf,
};

use crate::Result;

type Reader<T> = BufReader<ReadHalf<T>>;
type Writer<T> = BufWriter<WriteHalf<T>>;

pub async fn read_bytes<T: AsyncRead>(reader: &mut Reader<T>) -> Result<Vec<u8>> {
    let size = reader.read_u32_le().await? as usize;
    if size == 0 {
        return Ok(vec![]);
    }

    let mut buffer = vec![];
    buffer.resize(size, 0);

    reader.read_exact(&mut buffer).await?;

    Ok(buffer)
}

pub async fn write_bytes<T: AsyncWrite>(writer: &mut Writer<T>, bytes: &[u8]) -> Result<()> {
    let bytes_count = bytes.len() as u32;
    writer.write_all(&bytes_count.to_le_bytes()).await?;

    writer.write_all(&bytes).await?;
    writer.flush().await?;

    Ok(())
}

pub fn hexdump(data: &[u8]) {
    let mut width = 0;
    let mut count = 0;

    for chunk in data.chunks(16) {
        print!("{:08x}  ", count);
        count += chunk.len();

        for byte in chunk {
            print!("{:02x} ", byte);
            width += 1;
            if width == 8 {
                print!(" ");
            }
        }

        while width < 16 {
            print!("   ");
            width += 1;
            if width == 8 {
                print!(" ");
            }
        }

        print!(" |");

        for byte in chunk {
            if byte.is_ascii_alphanumeric() || byte.is_ascii_punctuation() {
                print!("{}", *byte as char);
            } else {
                print!(".");
            }
        }

        println!("|");

        width = 0;
    }

    println!()
}
