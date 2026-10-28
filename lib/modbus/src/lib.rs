use std::io;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

/// A Modbus TCP client. Each operation uses a fresh connection so a failed or
/// interrupted transaction cannot leave the next request out of sync.
pub struct Client {
    address: String,
    unit: u8,
}

impl Client {
    pub fn new(address: String, unit: u8) -> Self {
        Self { address, unit }
    }

    async fn request(&self, pdu: &[u8]) -> io::Result<Vec<u8>> {
        timeout(Duration::from_secs(2), self.exchange(pdu))
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "Modbus request timed out"))?
    }

    async fn exchange(&self, pdu: &[u8]) -> io::Result<Vec<u8>> {
        let mut stream = TcpStream::connect(&self.address).await?;
        let length = (pdu.len() + 1) as u16;
        let mut request = vec![0, 1, 0, 0, (length >> 8) as u8, length as u8, self.unit];
        request.extend_from_slice(pdu);
        stream.write_all(&request).await?;
        let mut header = [0; 7];
        stream.read_exact(&mut header).await?;
        let len = u16::from_be_bytes([header[4], header[5]]) as usize;
        if header[..4] != [0, 1, 0, 0] || header[6] != self.unit || !(2..=254).contains(&len) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid Modbus header",
            ));
        }
        let mut response = vec![0; len - 1];
        stream.read_exact(&mut response).await?;
        if response[0] == pdu[0] | 0x80 {
            return Err(io::Error::other(format!(
                "Modbus exception: {:?}",
                response.get(1)
            )));
        }
        if response[0] != pdu[0] {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected Modbus function",
            ));
        }
        Ok(response)
    }

    pub async fn read_coil(&self, address: u16) -> io::Result<bool> {
        let [hi, lo] = address.to_be_bytes();
        let response = self.request(&[1, hi, lo, 0, 1]).await?;
        match response.as_slice() {
            [1, 1, value] => Ok(value & 1 != 0),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid coil response",
            )),
        }
    }

    pub async fn read_holding_register(&self, address: u16) -> io::Result<u16> {
        let [hi, lo] = address.to_be_bytes();
        let response = self.request(&[3, hi, lo, 0, 1]).await?;
        match response.as_slice() {
            [3, 2, hi, lo] => Ok(u16::from_be_bytes([*hi, *lo])),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid register response",
            )),
        }
    }

    pub async fn write_coil(&self, address: u16, value: bool) -> io::Result<()> {
        self.write(5, address, if value { 0xff00 } else { 0 }).await
    }

    pub async fn write_holding_register(&self, address: u16, value: u16) -> io::Result<()> {
        self.write(6, address, value).await
    }

    async fn write(&self, function: u8, address: u16, value: u16) -> io::Result<()> {
        let [a, b] = address.to_be_bytes();
        let [c, d] = value.to_be_bytes();
        let request = [function, a, b, c, d];
        if self.request(&request).await? != request {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid write response",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn exchanges_and_validates_modbus_frames() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap().to_string();
        let server = tokio::spawn(async move {
            for (expected, response) in [
                (vec![1, 0, 0, 0, 1], vec![1, 1, 1]),
                (vec![3, 0, 2, 0, 1], vec![3, 2, 0, 3]),
                (vec![5, 0, 0, 0xff, 0], vec![5, 0, 0, 0xff, 0]),
                (vec![6, 0, 2, 0, 2], vec![0x86, 2]),
            ] {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut request = [0; 12];
                socket.read_exact(&mut request).await.unwrap();
                assert_eq!(&request[..7], &[0, 1, 0, 0, 0, 6, 1]);
                assert_eq!(&request[7..], expected);
                let len = (response.len() + 1) as u8;
                socket.write_all(&[0, 1, 0, 0, 0, len, 1]).await.unwrap();
                socket.write_all(&response).await.unwrap();
            }
        });
        let client = Client::new(address, 1);
        assert!(client.read_coil(0).await.unwrap());
        assert_eq!(client.read_holding_register(2).await.unwrap(), 3);
        client.write_coil(0, true).await.unwrap();
        assert!(client.write_holding_register(2, 2).await.is_err());
        server.await.unwrap();
    }
}
