use super::protocol;
use protocol::types::StreamId;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

// Stream keeps information about protocol stream with given id
struct Stream {
    id: StreamId,
    response_waker: Option<Waker>,
    state: StreamState,
}

enum StreamState {
    Free,
    Registered {
        register_semaphore_permit: OwnedSemaphorePermit,
    },
    Sent {
        register_semaphore_permit: OwnedSemaphorePermit,
    },
    SentButAbandoned {
        register_semaphore_permit: OwnedSemaphorePermit,
    },
    Responded {
        register_semaphore_permit: OwnedSemaphorePermit,
        response: protocol::Response,
    },
    Finished {
        register_semaphore_permit: OwnedSemaphorePermit,
    },
}

type SharedStream = Arc<std::sync::Mutex<Stream>>;

// StreamsManager coordinates assigning and freeing streams plus receiving messages
pub struct StreamsManager {
    streams: Vec<SharedStream>,
    free_streams: std::sync::Mutex<Vec<SharedStream>>,
    free_streams_semaphore: Arc<Semaphore>,
}

impl StreamsManager {
    pub fn new() -> Arc<StreamsManager> {
        let total_streams_possible: usize = (StreamId::max_value() as usize) + 1;

        let mut streams: Vec<SharedStream> = Vec::with_capacity(total_streams_possible);
        for i in 0..total_streams_possible {
            streams.push(Arc::new(std::sync::Mutex::new(Stream {
                id: i as StreamId,
                response_waker: None,
                state: StreamState::Free,
            })));
        }

        let mut free_streams: std::sync::Mutex<Vec<SharedStream>> =
            std::sync::Mutex::new(Vec::with_capacity(total_streams_possible));
        {
            let locked_free_streams: &mut Vec<SharedStream> = &mut free_streams.lock().unwrap();
            for stream in &streams {
                locked_free_streams.push(stream.clone());
            }
        }
        println!("Created free_streams size: {}", free_streams.lock().unwrap().len());


        return Arc::new(StreamsManager {
            streams,
            free_streams,
            free_streams_semaphore: Arc::new(Semaphore::new(total_streams_possible)),
        });
    }

    pub async fn register_stream(self: &Arc<Self>) -> StreamHandle {
        // Wait until a free stream is available
        let register_semaphore_permit: OwnedSemaphorePermit =
            self.free_streams_semaphore.clone().acquire_owned().await;

        // Take the stream and initialize it
        let the_stream: SharedStream = self.free_streams.lock().unwrap().pop().unwrap();
        {
            let locked_stream: &mut Stream = &mut the_stream.lock().unwrap();
            assert!(matches!(locked_stream.state, StreamState::Free));
            assert!(matches!(locked_stream.response_waker, None));

            locked_stream.state = StreamState::Registered {
                register_semaphore_permit,
            };
        }

        return StreamHandle {
            stream: the_stream,
            streams_manager: self.clone(),
        };
    }

    pub fn on_response_received(
        self: &Arc<Self>,
        response: protocol::Response,
        stream_id: StreamId,
    ) {
        let the_stream: &SharedStream = &self.streams[stream_id as usize];

        let mut waker_to_call: Option<Waker> = None;
        {
            let locked_stream: &mut Stream = &mut the_stream.lock().unwrap();
            let mut stream_state: StreamState = StreamState::Free;
            std::mem::swap(&mut stream_state, &mut locked_stream.state);

            match stream_state {
                StreamState::Sent {
                    register_semaphore_permit,
                } => {
                    locked_stream.state = StreamState::Responded {
                        register_semaphore_permit,
                        response,
                    };

                    waker_to_call = locked_stream.response_waker.take();
                }
                StreamState::SentButAbandoned {
                    register_semaphore_permit,
                } => {
                    // This stream has been abandoned by caller so let's just free it
                    locked_stream.response_waker = None;
                    self.free_streams.lock().unwrap().push(the_stream.clone());
                    // Semaphore register permit gets dropped here
                }
                _ => println!("Bad server response!"),
            };
        }

        if let Some(waker) = waker_to_call {
            waker.wake();
        }
    }

    pub fn on_receive_error(self: &Arc<Self>, error: std::io::Error) {
        unimplemented!();
    }
}

// StreamHandle is given to request maker to perform request on this stream
// If it's dropped then it's assumed that request maker doesn't need this stream anymore
pub struct StreamHandle {
    stream: SharedStream,
    streams_manager: Arc<StreamsManager>,
}

impl StreamHandle {
    pub fn get_stream_id(&self) -> StreamId {
        return self.stream.lock().unwrap().id;
    }

    pub fn mark_request_sent(&self) {
        let locked_stream: &mut Stream = &mut self.stream.lock().unwrap();
        let mut stream_state: StreamState = StreamState::Free;
        std::mem::swap(&mut stream_state, &mut locked_stream.state);

        if let StreamState::Registered {
            register_semaphore_permit,
        } = stream_state
        {
            stream_state = StreamState::Sent {
                register_semaphore_permit,
            };
        } else {
            unreachable!();
        }
    }

    pub fn get_response(self) -> StreamResponseFuture {
        return StreamResponseFuture {
            stream_handle: Some(self),
        };
    }
}

impl Drop for StreamHandle {
    fn drop(&mut self) {
        let locked_stream: &mut Stream = &mut self.stream.lock().unwrap();
        locked_stream.response_waker = None;

        let mut stream_state: StreamState = StreamState::Free;
        std::mem::swap(&mut stream_state, &mut locked_stream.state);

        match stream_state {
            StreamState::Registered {
                register_semaphore_permit,
            }
            | StreamState::Finished {
                register_semaphore_permit,
            } => {
                // Should be freed
                // Put on free queue
                self.streams_manager
                    .free_streams
                    .lock()
                    .unwrap()
                    .push(self.stream.clone());
                // Register semaphore permit gets dropped here
            }
            StreamState::Responded {
                register_semaphore_permit,
                response,
            } => {
                // Should be freed
                // Put on free queue
                self.streams_manager
                    .free_streams
                    .lock()
                    .unwrap()
                    .push(self.stream.clone());
                // Register semaphore permit gets dropped here
            }
            StreamState::Sent {
                register_semaphore_permit,
            } => {
                // Should be marked as Abandoned
                locked_stream.state = StreamState::SentButAbandoned {
                    register_semaphore_permit,
                };
            }
            StreamState::Free => unreachable!(),
            StreamState::SentButAbandoned {
                register_semaphore_permit,
            } => unreachable!(),
        };
    }
}

pub struct StreamResponseFuture {
    stream_handle: Option<StreamHandle>,
}

impl Future for StreamResponseFuture {
    type Output = Result<protocol::Response, std::io::Error>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let mut result: Option<protocol::Response> = None;

        if let Some(stream_handle) = &self.stream_handle {
            let locked_stream: &mut Stream = &mut stream_handle.stream.lock().unwrap();

            let mut stream_state: StreamState = StreamState::Free;
            std::mem::swap(&mut stream_state, &mut locked_stream.state);

            match stream_state {
                StreamState::Responded {
                    register_semaphore_permit,
                    response,
                } => {
                    result = Some(response);
                    locked_stream.state = StreamState::Finished {
                        register_semaphore_permit,
                    };
                }
                StreamState::Finished { .. } => unreachable!(),
                _ => {
                    locked_stream.response_waker = Some(context.waker().clone());
                }
            };
        } else {
            panic!("StreamResponseFuture polled after completion");
        }

        if let Some(response) = result {
            self.stream_handle = None; // Explicitly drop the handle
            return Poll::Ready(Ok(response));
        }

        return Poll::Pending;
    }
}
