use std::{collections::HashMap, fmt::Debug};

use crate::{Handler, Method};

pub struct RouterHandler {
    pub method: Method,
    pub path: String,
    pub handler: Handler,
}

impl Debug for RouterHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RouterHandler {{ method: {:?}, path: {}, handler: {:p} }}",
            self.method, self.path, &self.handler
        )
    }
}

impl RouterHandler {
    pub fn new(method: Method, path: &str, handler: Handler) -> Self {
        RouterHandler {
            method,
            path: path.to_string(),
            handler,
        }
    }
}

#[derive(Default)]
pub struct Router {
    // 实现 group
    group: HashMap<String, Router>,
    get: HashMap<String, Box<RouterHandler>>,
    post: HashMap<String, Box<RouterHandler>>,
    put: HashMap<String, Box<RouterHandler>>,
    delete: HashMap<String, Box<RouterHandler>>,
}

impl Debug for Router {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "get: {:?} \n post: {:?} \n put: {:?} \n delete: {:?} \n",
            self.get, self.post, self.put, self.delete
        )
    }
}

impl Router {
    /// 创建并返回一个Router实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 将route注册进Router中
    ///
    /// 1:使用闭包方式注册
    /// ```
    /// use httpx::{
    ///     Method,
    ///     HttpResuest,
    ///     HttpResponse,
    ///     Router, RouterHandler,
    ///     HttpServer,
    /// };
    /// let mut router = Router::new();
    /// router.get("/", |_r, w| {
    ///     w.write_str("hello world");
    /// });
    /// router.get("/hi", route_fn);
    /// fn route_fn(_r: &HttpResuest, w: &mut HttpResponse) {
    ///     w.insert_header("Content-Type", "text/html;charset=utf-8");
    ///     w.write_str("你好Rust");
    /// }
    /// ```
    pub fn get(&mut self, path: &str, handler: Handler) -> &Self {
        let h = RouterHandler::new(Method::GET, path, handler);
        self.insert(h);
        self
    }

    /// 将route注册进Router中
    ///
    /// 1:使用闭包方式注册
    /// ```
    /// use httpx::{
    ///     Method,
    ///     HttpResuest,
    ///     HttpResponse,
    ///     Router, RouterHandler,
    ///     HttpServer,
    /// };
    /// let mut router = Router::new();
    /// router.post("/", |_r, w| {
    ///     w.write_str("hello world");
    /// });
    /// router.post("/hi", route_fn);
    /// fn route_fn(_r: &HttpResuest, w: &mut HttpResponse) {
    ///     w.insert_header("Content-Type", "text/html;charset=utf-8");
    ///     w.write_str("你好Rust");
    /// }
    /// ```
    pub fn post(&mut self, path: &str, handler: Handler) -> &Self {
        let h = RouterHandler::new(Method::POST, path, handler);
        self.insert(h);
        self
    }

    /// 将route注册进Router中
    ///
    /// 1:使用闭包方式注册
    /// ```
    /// use httpx::{
    ///     Method,
    ///     HttpResuest,
    ///     HttpResponse,
    ///     Router, RouterHandler,
    ///     HttpServer,
    /// };
    /// let mut router = Router::new();
    /// router.put("/", |_r, w| {
    ///     w.write_str("hello world");
    /// });
    /// router.put("/hi", route_fn);
    /// fn route_fn(_r: &HttpResuest, w: &mut HttpResponse) {
    ///     w.insert_header("Content-Type", "text/html;charset=utf-8");
    ///     w.write_str("你好Rust");
    /// }
    /// ```
    pub fn put(&mut self, path: &str, handler: Handler) -> &Self {
        let h = RouterHandler::new(Method::PUT, path, handler);
        self.insert(h);
        self
    }

    /// 将route注册进Router中
    ///
    /// 1:使用闭包方式注册
    /// ```
    /// use httpx::{
    ///     Method,
    ///     HttpResuest,
    ///     HttpResponse,
    ///     Router, RouterHandler,
    ///     HttpServer,
    /// };
    /// let mut router = Router::new();
    /// router.delete("/", |_r, w| {
    ///     w.write_str("hello world");
    /// });
    /// router.delete("/hi", route_fn);
    /// fn route_fn(_r: &HttpResuest, w: &mut HttpResponse) {
    ///     w.insert_header("Content-Type", "text/html;charset=utf-8");
    ///     w.write_str("你好Rust");
    /// }
    /// ```
    pub fn delete(&mut self, path: &str, handler: Handler) -> &Self {
        let h = RouterHandler::new(Method::DELETE, path, handler);
        self.insert(h);
        self
    }

    fn insert(&mut self, h: RouterHandler) {
        let mut current = self;
        for part in h.path.split('/') {
            if part.is_empty() {
                continue;
            }

            current = current
                .group
                .entry(part.to_string())
                .or_insert_with(Router::new);
        }

        let k = h.path.clone();
        let handler = Box::new(RouterHandler::new(h.method, &h.path, h.handler));

        match handler.method {
            Method::GET => current.get.insert(k, handler),
            Method::POST => current.post.insert(k, handler),
            Method::PUT => current.put.insert(k, handler),
            Method::DELETE => current.delete.insert(k, handler),
            // Method::PATCH => todo!(),
            // Method::OPTIONS => todo!(),
            // Method::HEAD => todo!(),
        };
        // current.get = Some(handler);
    }

    pub fn get_handler<'a>(&'a self, method: Method, path: &'a str) -> Result<&RouterHandler, String> {
        let mut current = self;
        let mut params = HashMap::new();
        let mut _path = path.clone().to_string();

        for (i, part) in path.split('/').enumerate() {
            if part.is_empty() {
                continue;
            }

            if let Some(node) = current.group.get(part) {
                current = node;
            } else {
                // Check for dynamic route
                for (key, node) in current.group.iter() {
                    // println!("key: {}", key);
                    if key.starts_with(':') {
                        let mut list: Vec<&str> = _path.split("/").collect();
                        let i = list.partition_point(|v| v.ne(&part));
                        let s = String::from(key);
                        list[i] = &s;
                        _path = list.join("/");

                        // println!("{}", _path);

                        params.insert(&key[1..], part);
                        current = node;
                        break;
                    }
                }
            }

            if (i == path.split('/').count() - 1)
                && current.get.is_empty()
                && current.post.is_empty()
                && current.put.is_empty()
                && current.delete.is_empty()
            {
                return Err(format!("missing {} handler for path {}", method, path));
            }

            // if i == path.split('/').count() - 1 {
            //     match method {
            //         Method::GET => {
            //             if current.get.is_empty() {
            //                 return  None
            //             }
            //         },
            //         Method::POST => {
            //             if current.post.is_empty() {
            //                 return  None
            //             }
            //         },
            //         Method::PUT => {
            //             if current.put.is_empty() {
            //                 return  None
            //             }
            //         },
            //         Method::DELETE => {
            //             if current.delete.is_empty() {
            //                 return  None
            //             }
            //         },
            //     }
            // }

            // println!("{}, {}, {}, {}", current.get.is_none(), current.post.is_none(), current.put.is_none(), current.delete.is_none());

            // If there's no dynamic route and no match, return None
            // if i == path.split('/').count() - 1 && current.get.is_none() {
            //     return None;
            // }
        }

        match method {
            Method::GET => match current.get.get(&_path) {
                Some(h) => Ok(&h),
                None => return Err(format!("missing get handler for path {}", path)),
            },
            Method::POST => match current.post.get(&_path) {
                Some(h) => Ok(&h),
                None => return Err(format!("missing post handler for path {}", path)),
            },
            Method::PUT => match current.put.get(&_path) {
                Some(h) => Ok(&h),
                None => return Err(format!("missing put handler for path {}", path)),
            },
            Method::DELETE => match current.delete.get(&_path) {
                Some(h) => Ok(&h),
                None => return Err(format!("missing delete handler for path {}", path)),
            },
            // Method::PATCH => todo!(),
            // Method::OPTIONS => todo!(),
            // Method::HEAD => todo!(),
        }

        // if let Some(handler) = current.get {
        //     Some(handler)
        // } else {
        //     None
        // }
    }

    /// 将route注册进Router中
    ///
    /// 1:使用闭包方式注册
    /// ```
    /// use httpx::{
    ///     Method,
    ///     HttpResuest,
    ///     HttpResponse,
    ///     Router, RouterHandler,
    ///     HttpServer,
    /// };
    /// let mut router = Router::new();
    /// router.add_route(RouterHandler::new(Method::GET, "/", |_r, w| {
    ///     w.write_str("hello world");
    /// }));
    /// router.add_route(RouterHandler::new(Method::GET, "/hi", route_fn));
    /// fn route_fn(_r: &HttpResuest, w: &mut HttpResponse) {
    ///     w.insert_header("Content-Type", "text/html;charset=utf-8");
    ///     w.write_str("你好Rust");
    /// }
    /// ```
    pub fn add_route(&mut self, handler: RouterHandler) {
        match handler.method {
            Method::GET => self.get(&handler.path, handler.handler),
            Method::POST => self.post(&handler.path, handler.handler),
            Method::PUT => self.put(&handler.path, handler.handler),
            Method::DELETE => self.delete(&handler.path, handler.handler),
            // Method::PATCH => todo!(),
            // Method::OPTIONS => todo!(),
            // Method::HEAD => todo!(),
            // Method::NoSupport => &Router::from(Router::new()),
        };
    }

}

impl From<Router> for String {
    fn from(_r: Router) -> Self {
        let body_str = format!("NoSupport \r\n");
        return body_str;
    }
}

// pub struct Router {
//     routes: Vec<Box<RouterHandler>>,
//     _routes_cache: HashMap<String, Box<RouterHandler>>,
// }

// impl Router {
//     /// 创建并返回一个Router实例
//     pub fn new() -> Self {
//         Router {
//             routes: Vec::new(),
//             _routes_cache: HashMap::new(),
//         }
//     }

//     /// 将route注册进Router中
//     ///
//     /// 1:使用闭包方式注册
//     /// ```
//     /// use http::{
//     ///     http_method::method::Method,
//     ///     http_request::request::HttpResuest,
//     ///     http_response::response::HttpResponse,
//     ///     http_router::{router::Router, router_handler::RouterHandler},
//     ///     http_server::server::HttpServer,
//     /// };
//     /// let mut router = Router::new();
//     /// router.add_route(RouterHandler::new(Method::GET, "/", |_r, w| {
//     ///     w.write_str("hello world");
//     /// }));
//     /// router.add_route(RouterHandler::new(Method::GET, "/hi", route_fn));
//     /// fn route_fn(_r: &HttpResuest, w: &mut HttpResponse) {
//     ///     w.insert_header("Content-Type", "text/html;charset=utf-8");
//     ///     w.write_str("你好Rust");
//     /// }
//     /// ```
//     pub fn add_route(&mut self, handler: RouterHandler) {
//         let paths = Self::split_path(path);
//         self.routes.push(Box::new(handler));
//     }

//     pub(crate) fn get_handler(&self, method: Method, path: &str) -> Option<&RouterHandler> {
//         // 首先查询缓存，命中则返回，否则进行匹配
//         match self._routes_cache.get(&format!("{}/{}", method, path)) {
//             Some(handler) => Some(handler),
//             None => {
//                 if let Some(router_handler) = self.get_handler_vec(method, path) {
//                     return Some(&router_handler);
//                 } else {
//                     return None;
//                 }
//             }
//         }
//     }

//     fn get_handler_vec(&self, method: Method, path: &str) -> Option<&RouterHandler> {
//         for route in self.routes.iter() {
//             if route.method == method && route.path == path {
//                 return Some(route);
//             }
//         }
//         None
//     }
// }
