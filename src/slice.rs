use crate::core;
use prost;
use prost::bytes::BytesMut;
use prost::Message;

fn fetch(
    requester: &zmq::Socket,
    guid: &str,
) -> Result<core::FetchResponse, zmq::Error> {
    let mut req = core::FetchRequest {
        guid: guid.to_string(),
        root: "".to_string(),
        function: None,
        ids: vec![],
        requestid: "".to_string(),
        shape: None,
    };

    let mut buf = BytesMut::with_capacity(10);
    req.encode(&mut buf).unwrap();
    requester.send(&buf[..], 0)?;

    let msg = requester.recv_msg(0)?;
    Ok(core::FetchResponse::decode(&msg[..]).unwrap())
}

fn get_slice(
    requester: &zmq::Socket,
    guid: &str,
) -> Result<core::SliceResponse, zmq::Error> {
    fetch(requester, guid);
    let slice = core::SliceResponse::default();
    Ok(slice)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    fn _mock_response() -> core::FetchResponse {
        core::FetchResponse {
            function: None,
            requestid: "".to_string(),
        }
    }

    #[test]
    fn slice_zmq() {
        let ctx = zmq::Context::new();
        let addr = "inproc://slice_zmq";

        let responder = ctx.socket(zmq::REP).unwrap();
        responder.bind(addr).unwrap();
        let mst = thread::spawn(move || {
            mock_serve(&responder);
        });

        let requester = ctx.socket(zmq::REQ).unwrap();
        requester.connect(addr).unwrap();

        let _s = get_slice(&requester, "guid").unwrap();
        // assert_eq!(slice.lineno, s.lineno);
        mst.join().unwrap();
    }

    fn mock_serve(responder: &zmq::Socket) {
        let b = responder.recv_bytes(0).unwrap();
        let fr = core::FetchRequest::decode(&b[..]).unwrap();

        let fetchResponse = core::FetchResponse::default();
        let mut buf = BytesMut::with_capacity(10);
        fetchResponse.encode(&mut buf).unwrap();
        responder.send(&buf[..], 0);
    }
}
