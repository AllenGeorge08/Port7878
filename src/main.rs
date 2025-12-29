use multi_threaded_server_rust::ThreadPool;
use std::{
    fs, io::{BufRead, BufReader, Write}, net::{TcpListener, TcpStream}, thread, time::Duration
};

fn main() {
    // let listener = TcpListener::bind("127.0.0.1.7878").unwrap();
     let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4); //e To limit the number of threads being created

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        
        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Hello, world!");
}

//to read data from the tcp stream:
fn handle_connection(mut stream: TcpStream) {
    //e takes http stream as a request..
    let buf_reader = BufReader::new(&stream);
    // let http_request: Vec<_> = buf_reader
    //     .lines()
    //     .map(|result| result.unwrap())
    //     .take_while(|line| !line.is_empty())
    //     .collect();

    // println!("Request: {http_request:#?}");

    // `BufReader<R>` can improve the speed of programs that make *small* and
    // / *repeated* read calls to the same file or network socket. It does not
    // / help when reading very large amounts at once, or reading just one or a few
    // / times. It also provides no advantage when reading from a source that is
    // / already in memory, like a <code>[Vec]\<u8></code>.
    // /
    // / When the `BufReader<R>` is dropped, the contents of its buffer will be
    // / discarded. Creating multiple instances of a `BufReader<R>` on the same
    // / stream can cause data loss. Reading from the underlying reader after
    // / unwrapping the `BufReader<R>` with [`BufReader::into_inner`] can also cause
    // / data loss.

    let request_line = buf_reader.lines().next().unwrap().unwrap(); //e we only need the first line of the http request, so rather than reading the entire request into a vector, we're calling next to get the frst item from the iterator. The first unwrap takes care of the option and stops the program if the iterator has no items.

    let (status_line,filename) = match &request_line[..]{
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 404 NOT FOUND", "404.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();

}