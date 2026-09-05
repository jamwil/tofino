use super::http::{HttpResponse, HttpVersion};
use super::{BuildResource, CreateRequest, ResponseParseError};
use crate::resources::{Resource, WebResource};
use native_tls::TlsConnector;
use std::collections::HashMap;
use std::error::Error;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::FromStr;

pub struct HttpsRequest<'a> {
    pub version: HttpVersion,
    pub host: &'a str,
    pub port: usize,
    pub path: &'a str,
}

impl<'a> CreateRequest for HttpsRequest<'a> {
    fn get(&self) -> Result<impl BuildResource, Box<dyn Error>> {
        // Construct the request
        let socket_addr = self.host.to_string() + ":" + &self.port.to_string();
        let request = format!(
            "GET {} {}\r\nHost: {}\r\n\r\n",
            &self.path, &self.version, &self.host
        );

        // Create a TCP socket connection and send the request
        let mut buffer = String::new();
        let connector = TlsConnector::new().unwrap();
        let tcp_stream = TcpStream::connect(socket_addr)?;
        let mut stream = connector.connect(self.host, tcp_stream).unwrap();
        stream.write_all(&request.into_bytes())?;
        stream.read_to_string(&mut buffer)?;

        // Parse the response or propogate the error
        Ok(HttpsResponse::from_str(&buffer)?)
    }
}

pub struct HttpsResponse {
    pub version: HttpVersion,
    pub status: String,
    pub explanation: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl FromStr for HttpsResponse {
    type Err = ResponseParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match HttpResponse::from_str(s) {
            Ok(r) => Ok(Self {
                version: r.version,
                status: r.status,
                explanation: r.explanation,
                headers: r.headers,
                body: r.body,
            }),
            Err(e) => Err(e),
        }
    }
}

impl BuildResource for HttpsResponse {
    fn build_resource(&self) -> Resource {
        Resource::Web(WebResource {
            response_status: self.status.clone(),
            response_explanation: self.explanation.clone(),
            response_headers: self.headers.clone(),
            response_body: self.body.clone(),
        })
    }
}
