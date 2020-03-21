use crate::core;
use prost::bytes::BytesMut;
use prost::Message;

struct Fetcher {
    requester: zmq::Socket,
}

impl Fetcher {
    fn fetch(self: &Self, buf: &BytesMut) -> Result<Vec<u8>, zmq::Error> {
        self.requester.send(&buf[..], 0)?;
        self.requester.recv_bytes(0)
    }

    fn get_slice(
        self: &Self,
        req: &core::ApiRequest,
    ) -> Result<core::SliceResponse, FetchError> {
        let mut buf = BytesMut::with_capacity(10);
        req.encode(&mut buf)?;

        match core::FetchResponse::decode(&self.fetch(&buf)?[..])?.function {
            Some(core::fetch_response::Function::Slice(s)) => Ok(s),
            _ => Err(FetchError::MissingResponse),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn slice_zmq() {
        let ctx = zmq::Context::new();
        let addr = "inproc://slice_zmq";

        let responder = ctx.socket(zmq::REP).unwrap();
        responder.bind(addr).unwrap();
        let mock_thread = thread::spawn(move || {
            mock_serve(&responder);
        });

        let requester = ctx.socket(zmq::REQ).unwrap();
        requester.connect(addr).unwrap();

        let fetcher = Fetcher {
            requester: requester,
        };

        let s: core::SliceResponse =
            fetcher.get_slice(&core::ApiRequest::default()).unwrap();
        assert_eq!(vec![core::SliceTile::default()], s.tiles);

        mock_thread.join().unwrap();
    }

    fn mock_serve(responder: &zmq::Socket) {
        let mut buf = BytesMut::with_capacity(10);
        let rid = core::ApiRequest::decode(&responder.recv_bytes(0).unwrap()[..])
            .unwrap()
            .requestid;
        core::FetchResponse {
            requestid: rid,
            function: Some(core::fetch_response::Function::Slice(
                core::SliceResponse {
                    layout: None,
                    tiles: vec![core::SliceTile::default()],
                },
            )),
        }
        .encode(&mut buf)
        .unwrap();
        responder.send(&buf[..], 0).unwrap();
    }
}

#[derive(Debug)]
enum FetchError {
    ZmqError(zmq::Error),
    DecodeError(prost::DecodeError),
    EncodeError(prost::EncodeError),
    MissingResponse,
}

impl From<zmq::Error> for FetchError {
    fn from(err: zmq::Error) -> FetchError {
        FetchError::ZmqError(err)
    }
}

impl From<prost::DecodeError> for FetchError {
    fn from(err: prost::DecodeError) -> FetchError {
        FetchError::DecodeError(err)
    }
}

impl From<prost::EncodeError> for FetchError {
    fn from(err: prost::EncodeError) -> FetchError {
        FetchError::EncodeError(err)
    }
}
