use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
};

use num_cpus;

use crate::{ThreadPool, HttpRequest, HttpResponse, HttpStateCode, Router};

pub(crate) trait Executor {
    fn executor(&self, stream: TcpStream);
}

#[derive(Debug)]
pub struct HttpServer {
    addr: String,
    router: Arc<Router>,
    pool: ThreadPool,
    response: HttpResponse,
}

impl HttpServer {
    /// 返回一个HttpServer实例
    pub fn application() -> Self {
        HttpServer::default()
    }

    /// 设置HttpServer参数
    pub fn configure<F>(&mut self, opt: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        opt(self);
        self
    }

    pub fn mount_route(&mut self, route: Router) -> &mut Self {
        self.router = Arc::new(route);
        self
    }

    pub fn mount_header(&mut self, key: &str, value: &str) -> &mut Self {
        self.response.insert_header(key, value);
        // header.insert("Access-Control-Allow-Origin".to_string(), "*".to_owned());
        self
    }

    /// 启动Http服务
    pub fn start(&self) {
        let listener = TcpListener::bind(&self.addr).unwrap();
        println!("http server start at {}", self.addr);
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    // stream只对当前请求有效，故在此可转移所有权而非借用
                    self.executor(stream);
                }
                Err(_e) => {
                    println!("start err: {}", _e);
                    continue;
                }
            }
        }
    }

    /// 设置HttpServer监听地址，默认值："127.0.0.1:8080"
    pub fn set_addr(addr: &str) -> impl FnOnce(&mut HttpServer) {
        // 不可直接捕获参数所有权
        let a = addr.to_owned();
        |t: &mut Self| {
            t.addr = a;
        }
    }

    fn executor(&self, mut stream: TcpStream) {
        // println!("process stream");
        let router = self.router.clone();
        // let request = Self::parse_stream(&mut stream);
        let mut resp = HttpResponse::from(self.response.clone());

        // match router.get_handler(request.method, &request.uri) {
        //     Ok(s) => {
        //         // println!("{}", "executor");
        //         resp.set_http_state_code(HttpStateCode::StatusOK);
        //         let handler = s.handler;
        //         handler(&request, &mut resp);
        //     }
        //     Err(e) => {
        //         println!("err: {}", e);
        //         // HttpResponse::new().write_str(&e);
        //         // return;
        //     }
        // }

        // let resp_str: String = resp.into();
        // if let Err(e) = stream.write_all(resp_str.as_bytes()) {
        //     println!("response write error: {}", e);
        // }
        self.pool.execute(move || {
            let request = Self::parse_stream(&mut stream);
            // let mut resp = HttpResponse::default();

            match router.get_handler(request.method, &request.uri) {
                Ok(s) => {
                    // println!("{}", "executor");
                    resp.set_http_state_code(HttpStateCode::StatusOK);
                    let handler = s.handler;
                    handler(&request, &mut resp);
                }
                Err(e) => {
                    println!("err: {}", e);
                    // HttpResponse::new().write_str(&e);
                    // return;
                }
            }

            let resp_str: String = resp.into();
            if let Err(e) = stream.write_all(resp_str.as_bytes()) {
                println!("response write error: {}", e);
            }
        });
    }

    fn parse_stream(stream: &mut TcpStream) -> HttpRequest {
        let mut buf: Vec<u8> = Vec::new();
        let _len = Self::parse_stream_to_request(stream, &mut buf);
        let mut request = HttpRequest::from(String::from_utf8_lossy(buf.as_slice()).to_string());
        request.set_remote_addr(stream.peer_addr().unwrap().to_string().as_str());
        // s.router.get_handler(request.method, &request.uri);
        request
    }

    // 读取http请求信息，这个函数可能存在一些bug，比如读取到的数据不完整，需要继续读取
    fn parse_stream_to_request(stream: &mut TcpStream, buf: &mut Vec<u8>) -> usize {
        let mut req = [0; 1024];
        let mut pack_len: usize = 0;
        // println!("remote ip : {}", stream.peer_addr().unwrap().to_string());
        while let Ok(len) = stream.read(&mut req) {
            // println!("pack len : {}", len);
            if len == 0 {
                break;
            }
            pack_len += len;
            buf.extend_from_slice(&req[..len]);
            if len < req.len() {
                break;
            }
        }
        pack_len
    }

    fn default() -> Self {
        let cpu_num = num_cpus::get();
        HttpServer {
            addr: "127.0.0.1:8080".parse().unwrap(),
            router: Arc::new(Router::new()),
            pool: ThreadPool::new(cpu_num + 1),
            response: HttpResponse::default(),
        }
    }
}

// #[cfg(not(feature = "thread-pool"))]
// #[derive(Debug)]
// pub struct HttpServer {
//     addr: String,
//     router: Arc<Router>,
// }

// #[cfg(feature = "thread-pool")]
// #[derive(Debug)]
// pub struct HttpServer {
//     addr: String,
//     router: Arc<Router>,
//     pool: thread_pool::ThreadPool,
// }

// impl HttpServer {
//     /// 返回一个HttpServer实例
//     pub fn application() -> Self {
//         HttpServer::default()
//     }

//     /// 设置HttpServer参数
//     pub fn configure<F>(&mut self, opt: F) -> &mut Self
//     where
//         F: FnOnce(&mut Self),
//     {
//         opt(self);
//         self
//     }

//     pub fn mount_route(&mut self, route: Router) -> &mut Self {
//         self.router = Arc::new(route);
//         self
//     }

//     /// 启动Http服务
//     pub fn start(&self) {
//         let listener = TcpListener::bind(&self.addr).unwrap();
//         println!("http server start at {}", self.addr);
//         for stream in listener.incoming() {
//             match stream {
//                 Ok(stream) => {
//                     // stream只对当前请求有效，故在此可转移所有权而非借用
//                     self.executor(stream);
//                 }
//                 Err(_e) => {
//                     continue;
//                 }
//             }
//         }
//     }
// }

// impl HttpServer {
//     /// 设置HttpServer监听地址，默认值："127.0.0.1:8080"
//     pub fn set_addr(addr: &str) -> impl FnOnce(&mut HttpServer) {
//         // 不可直接捕获参数所有权
//         let a = addr.to_owned();
//         |t: &mut Self| {
//             t.addr = a;
//         }
//     }
// }

// #[cfg(feature = "thread-pool")]
// impl HttpServer {
//     /// 设置线程池大小，默认线程数：cpu核数 + 1
//     pub fn set_thread_pool_num(num: usize) -> impl FnOnce(&mut HttpServer) {
//         let n = num;
//         // 加入Move强制转移所有权，否则n的生命周期不够长
//         move |t: &mut Self| {
//             t.pool = thread_pool::ThreadPool::new(n);
//         }
//     }
// }

// #[cfg(not(feature = "thread-pool"))]
// impl Executor for HttpServer {
//     fn executor(&self, mut stream: TcpStream) {
//         use std::thread;
//         let router = self.router.clone();
//         thread::spawn(move || {
//             // println!("process stream");
//             let request = Self::parse_stream(&mut stream);
//             let mut resp = HttpResponse::default();
//             if let Some(handler_wrap) = router.get_handler(request.method, &request.uri) {
//                 let handler = handler_wrap.handler;
//                 handler(&request, &mut resp);
//             }
//             let resp_str: String = resp.into();
//             if let Err(e) = stream.write_all(resp_str.as_bytes()) {
//                 println!("response write error: {}", e);
//             }
//         });
//         // println!("process stream by not thread-pool");
//     }
// }

// #[cfg(feature = "thread-pool")]
// impl Executor for HttpServer {
//     fn executor(&self, mut stream: TcpStream) {
//         // println!("process stream");
//         let router = self.router.clone();
//         self.pool.execute(move || {
//             let request = Self::parse_stream(&mut stream);
//             let mut resp = HttpResponse::default();
//             if let Some(handler_wrap) = router.get_handler(request.method, &request.uri) {
//                 resp.set_http_state_code(HttpStateCode::StatusOK);
//                 let handler = handler_wrap.handler;
//                 handler(&request, &mut resp);
//             }
//             let resp_str: String = resp.into();
//             if let Err(e) = stream.write_all(resp_str.as_bytes()) {
//                 println!("response write error: {}", e);
//             }
//         });
//     }
// }

// impl HttpServer {
//     fn parse_stream(stream: &mut TcpStream) -> HttpRequest {
//         let mut buf: Vec<u8> = Vec::new();
//         let _len = Self::parse_stream_to_request(stream, &mut buf);
//         let mut request = HttpRequest::from(String::from_utf8_lossy(buf.as_slice()).to_string());
//         request.set_remote_addr(stream.peer_addr().unwrap().to_string().as_str());
//         // s.router.get_handler(request.method, &request.uri);
//         request
//     }

//     // 读取http请求信息，这个函数可能存在一些bug，比如读取到的数据不完整，需要继续读取
//     fn parse_stream_to_request(stream: &mut TcpStream, buf: &mut Vec<u8>) -> usize {
//         let mut req = [0; 1024];
//         let mut pack_len: usize = 0;
//         // println!("remote ip : {}", stream.peer_addr().unwrap().to_string());
//         while let Ok(len) = stream.read(&mut req) {
//             // println!("pack len : {}", len);
//             if len == 0 {
//                 break;
//             }
//             pack_len += len;
//             buf.extend_from_slice(&req[..len]);
//             if len < req.len() {
//                 break;
//             }
//         }
//         pack_len
//     }
// }

// #[cfg(not(feature = "thread-pool"))]
// impl Default for HttpServer {
//     fn default() -> Self {
//         HttpServer {
//             addr: "127.0.0.1:8080".parse().unwrap(),
//             router: Arc::new(Router::new()),
//         }
//     }
// }

// #[cfg(feature = "thread-pool")]
// impl Default for HttpServer {
//     fn default() -> Self {
//         let cpu_num = num_cpus::get();
//         HttpServer {
//             addr: "127.0.0.1:8080".parse().unwrap(),
//             router: Arc::new(Router::new()),
//             pool: thread_pool::ThreadPool::new(cpu_num + 1),
//         }
//     }
// }
