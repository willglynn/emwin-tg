use futures::StreamExt;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::*;

#[wasm_bindgen]
pub struct TextStream {
    buffer: Rc<Mutex<VecDeque<Result<crate::Product, crate::Error>>>>,
}

#[wasm_bindgen]
impl TextStream {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let buffer = Rc::new(Mutex::new(VecDeque::with_capacity(50)));

        let inner_buffer = buffer.clone();
        spawn_local(async move {
            let mut stream = crate::TextStream::default();
            while let Some(result) = stream.next().await {
                inner_buffer.lock().unwrap().push_back(result);
            }
        });

        Self { buffer }
    }

    #[wasm_bindgen]
    pub fn pop(&mut self) -> Result<Option<Product>, JsError> {
        match self.buffer.lock().unwrap().pop_front() {
            Some(Ok(v)) => Ok(Some(v.into())),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }
}

#[wasm_bindgen]
pub struct Product(crate::Product);

#[wasm_bindgen]
impl Product {
    #[wasm_bindgen(getter)]
    pub fn filename(&self) -> String {
        self.0.filename.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn string_contents(&self) -> String {
        self.0.string_contents().into_owned()
    }

    #[wasm_bindgen(getter)]
    pub fn mime_type(&self) -> Option<String> {
        self.0.mime_type().map(String::from)
    }
}

impl From<crate::Product> for Product {
    fn from(p: crate::Product) -> Self {
        Self(p)
    }
}

impl From<crate::Product> for JsValue {
    fn from(p: crate::Product) -> Self {
        Product::from(p).into()
    }
}

impl From<crate::Error> for JsValue {
    fn from(e: crate::Error) -> Self {
        JsError::from(e).into()
    }
}
