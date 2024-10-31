use std::num::NonZeroUsize;

use accessibility_ng::{AXAttribute, AXUIElement};
use accessibility_sys_ng::{kAXFocusedUIElementAttribute, kAXSelectedTextAttribute};
use active_win_pos_rs::get_active_window;
use core_foundation::string::CFString;
use debug_print::debug_println;
use enigo::*;
use lru::LruCache;
use parking_lot::Mutex;

static GET_SELECTED_TEXT_METHOD: Mutex<Option<LruCache<String, u8>>> = Mutex::new(None);

pub fn get_selected_text() -> Result<String, Box<dyn std::error::Error>> {
    if GET_SELECTED_TEXT_METHOD.lock().is_none() {
        let cache = LruCache::new(NonZeroUsize::new(100).unwrap());
        *GET_SELECTED_TEXT_METHOD.lock() = Some(cache);
    }
    let mut cache = GET_SELECTED_TEXT_METHOD.lock();
    let cache = cache.as_mut().unwrap();
    let app_name = match get_active_window() {
        Ok(window) => window.app_name,
        Err(_) => return Err("No active window found".into()),
    };
    // debug_println!("app_name: {}", app_name);
    if let Some(text) = cache.get(&app_name) {
        if *text == 0 {
            return get_selected_text_by_ax();
        }
        return get_selected_text_by_clipboard();
    }
    match get_selected_text_by_ax() {
        Ok(text) => {
            if !text.is_empty() {
                cache.put(app_name, 0);
            }
            Ok(text)
        }
        Err(_) => match get_selected_text_by_clipboard() {
            Ok(text) => {
                if !text.is_empty() {
                    cache.put(app_name, 1);
                }
                Ok(text)
            }
            Err(e) => Err(e),
        },
    }
}

fn get_selected_text_by_ax() -> Result<String, Box<dyn std::error::Error>> {
    debug_println!("get_selected_text_by_ax");
    let system_element = AXUIElement::system_wide();
    let Some(selected_element) = system_element
        .attribute(&AXAttribute::new(&CFString::from_static_string(
            kAXFocusedUIElementAttribute,
        )))
        .map(|element| element.downcast_into::<AXUIElement>())
        .ok()
        .flatten()
    else {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No selected element",
        )));
    };
    let Some(selected_text) = selected_element
        .attribute(&AXAttribute::new(&CFString::from_static_string(
            kAXSelectedTextAttribute,
        )))
        .map(|text| text.downcast_into::<CFString>())
        .ok()
        .flatten()
    else {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No selected text",
        )));
    };
    Ok(selected_text.to_string())
}

pub fn get_selected_text_by_clipboard() -> Result<String, Box<dyn std::error::Error>> {
    debug_println!("get_selected_text_by_clipboard");
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    crate::utils::get_selected_text_by_clipboard(&mut enigo, false)
}
