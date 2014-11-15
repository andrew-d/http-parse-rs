#![feature(macro_rules)]


macro_rules! go {
    /*
    ( $me:expr : $a:tt                   ; $($rest:tt)* ) => ({ });
    ( $me:expr : $a:tt $b:tt             ; $($rest:tt)* ) => ({ });
    ( $me:expr : $a:tt $b:tt $c:tt       ; $($rest:tt)* ) => ({ });
    ( $me:expr : $a:tt $b:tt $c:tt $d:tt ; $($rest:tt)* ) => ({ });
    */

    // State transitions with one or two arguments
    ( $me:expr : to $s:ident                       ) => ({ $me.state = states::$s;               return true; });
    ( $me:expr : to $s:ident $arg1:expr            ) => ({ $me.state = states::$s($arg1);        return true; });
    ( $me:expr : to $s:ident $arg1:expr $arg2:expr ) => ({ $me.state = states::$s($arg1($arg2)); return true; });

    // Re-consume with this current byte
    ( $me:expr : reconsume $s:ident                       ) => ({ $me.reconsume = true; go!($me: to $s);             });
    ( $me:expr : reconsume $s:ident $arg1:expr            ) => ({ $me.reconsume = true; go!($me: to $s $arg1);       });
    ( $me:expr : reconsume $s:ident $arg1:expr $arg2:expr ) => ({ $me.reconsume = true; go!($me: to $s $arg1 $arg2); });

    // Mark a value
    ( $me:expr : mark $mname:expr ) => ({ /* TODO */ });
}


const CR: u8 = b'\r';
const LF: u8 = b'\n';


// Helpful utilities
mod util;


enum ParseState {
    // Response line parsing
    RespStart,
    RespH,
    RespHT,
    RespHTT,
    RespHTTP,
    RespFirstHttpMajor,
    RespHttpMajor,
    RespFirstHttpMinor,
    RespHttpMinor,
    RespFirstStatusCode,
    RespStatusCode,
    RespStatusStart,
    RespStatus,
    RespLineAlmostDone,

    // Request line parsing
    ReqStart,
    ReqMethod,
    ReqSpacesBeforeUrl,
    ReqSchema,
    ReqSchemaSlash,
    ReqSchemaSlashSlash,
    ReqServerStart,
    ReqServer,
    ReqServerWithAt,
    ReqPath,
    ReqQueryStringStart,
    ReqQueryString,
    ReqFragmentStart,
    ReqFragment,
    ReqHttpStart,
    ReqH,
    ReqHT,
    ReqHTT,
    ReqHTTP,
    ReqFirstHttpMajor,
    ReqHttpMajor,
    ReqFirstHttpMinor,
    ReqHttpMinor,
    ReqLineAlmostDone,

    // Header parsing
    HeaderFieldStart,
    HeaderField,
    HeaderValueDiscardWs,
    HeaderValueDiscardWsAlmostDone,
    HeaderValueDiscardLWs,
    HeaderValueStart,
    HeaderValue,
    HeaderValueLWs,
    HeaderAlmostDone,

    // TODO: Chunked encoding support?
}


/// Callbacks that are triggered when a certain condition occurs.
pub enum NotifyCallbacks {
    OnMessageBegin,
    OnHeadersComplete,
    OnMessageComplete,
}


/// The type of a notification callback.  Should return 'false' to indicate
/// an error.
pub type NotifyCallback<'a> = ||: 'a -> bool;


/// Callbacks that are triggered when a certain type of data is encountered.
pub enum DataCallbacks {
    OnUrl,
    OnStatus,
    OnHeaderField,
    OnHeaderValue,
    OnBody,
}


/// The type of a data callback.  Should return 'false' to indicate an error.
pub type DataCallback<'a> = |&[u8]|: 'a -> bool;


// Wrapper for callbacks
struct HttpParserCallbacks<'a> {
    on_message_begin:    Option<NotifyCallback<'a>>,
    on_url:              Option<DataCallback<'a>>,
    on_status:           Option<DataCallback<'a>>,
    on_header_field:     Option<DataCallback<'a>>,
    on_header_value:     Option<DataCallback<'a>>,
    on_headers_complete: Option<NotifyCallback<'a>>,
    on_body:             Option<DataCallback<'a>>,
    on_message_complete: Option<NotifyCallback<'a>>,
}


impl<'a> HttpParserCallbacks<'a> {
    fn new<'n>() -> HttpParserCallbacks<'n> {
        HttpParserCallbacks {
            on_message_begin:    None,
            on_url:              None,
            on_status:           None,
            on_header_field:     None,
            on_header_value:     None,
            on_headers_complete: None,
            on_body:             None,
            on_message_complete: None,
        }
    }

    fn set_notify_cb(&mut self, cb: NotifyCallbacks, f: NotifyCallback<'a>) {
        match cb {
            OnMessageBegin =>    { self.on_message_begin    = Some(f) },
            OnHeadersComplete => { self.on_headers_complete = Some(f) },
            OnMessageComplete => { self.on_message_complete = Some(f) },
        };
    }

    // TODO: this shouldn't need to be `&mut self`
    #[inline]
    fn call_notify_cb(&mut self, cb: NotifyCallbacks) -> bool {
        let f = match cb {
            OnMessageBegin =>    { &mut self.on_message_begin },
            OnHeadersComplete => { &mut self.on_headers_complete },
            OnMessageComplete => { &mut self.on_message_complete },
        };

        match f {
            &Some(ref mut f) => (*f)(),
            &None            => true,
        }
    }

    fn set_data_cb(&mut self, cb: DataCallbacks, f: DataCallback<'a>) {
        match cb {
            OnUrl         => { self.on_url          = Some(f) },
            OnStatus      => { self.on_status       = Some(f) },
            OnHeaderField => { self.on_header_field = Some(f) },
            OnHeaderValue => { self.on_header_value = Some(f) },
            OnBody        => { self.on_body         = Some(f) },
        };
    }

    // TODO: this shouldn't need to be `&mut self`
    #[inline]
    fn call_data_cb(&mut self, cb: DataCallbacks, data: &[u8]) -> bool {
        let f = match cb {
            OnUrl         => { &mut self.on_url },
            OnStatus      => { &mut self.on_status },
            OnHeaderField => { &mut self.on_header_field },
            OnHeaderValue => { &mut self.on_header_value },
            OnBody        => { &mut self.on_body },
        };

        match f {
            &Some(ref mut f) => (*f)(data),
            &None            => true,
        }
    }
}


pub struct HttpParser<'a> {
    state: ParseState,
    reconsume: bool,

    // List of callbacks that we could execute.
    cbs: HttpParserCallbacks<'a>,
}

impl<'a> HttpParser<'a> {
    /// Create a new HttpParser
    pub fn new<'n>(is_request: bool) -> HttpParser<'n> {
        let start_state = if is_request {
            ReqStart
        } else {
            RespStart
        };

        HttpParser {
            state: start_state,
            reconsume: false,
            cbs: HttpParserCallbacks::new(),
        }
    }

    /// Feed more data to the parser.  The `is_eof` argument should be set when
    /// this data is the end of the request.  It is legal to pass an empty
    /// slice when `is_eof` is true.
    pub fn process(&mut self, data: &[u8], is_eof: bool) {
        // Handle the 0-length data case
        if data.len() == 0 {
            // TODO:
            return;
        }

        let mut it = data.iter();
        loop {
            let ch = match it.next() {
                Some(ch) => *ch,
                None     => break,
            };

            match self.state {
                RespStart => match ch {
                    b'\r' | b'\n' => break,
                    b'H' => {
                        self.state = RespH;
                        continue;
                    },
                    _ => { /* TODO ERROR */ },
                },

                RespH => match ch {
                    b'T' => { self.state = RespHT; continue; },
                    _    => break,
                },

                // TODO: remove me when we're exhaustive
                _ => panic!("unknown state"),
            };
        }
    }

    /// Register a notification callback.
    pub fn on(&mut self, cb: NotifyCallbacks, f: NotifyCallback<'a>) {
        self.cbs.set_notify_cb(cb, f)
    }

    /// Register a data callback.
    pub fn on_data(&mut self, cb: DataCallbacks, f: DataCallback<'a>) {
        self.cbs.set_data_cb(cb, f)
    }
}


#[test]
fn test_can_construct() {
    let _ = HttpParser::new(true);
    let _ = HttpParser::new(false);
}
