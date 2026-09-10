use std::{error::Error} ;

//use std::str::FromStr;

use tokio::time::Duration;

use strum_macros::{
        //Display,
        AsRefStr
    };

use playhard_launcher::LaunchOptions ;

use playhard_automation::LoadState;

use log::{
        error, 
        info
    } ;

use serde::{
        //Deserialize, 
        Serialize
    };

use crate::my_browser ;

// url получения tor bridge
const URL_BRIDGE: &str = "https://bridges.torproject.org/options" ;

// прокси для работы с браузером
pub const PROXY_URL: &str = "socks5://127.0.0.1:9150" ;

// css selector поиска кнопки на странице
const CSS_BUTTON: &str = r#"form > input[type="submit"]"# ;

// css selector поиска поля выбора типов bridge
const CSS_TYPE_BRIDGE: &str = "select#advanced-options-transport" ;

// css selector поиска поля IPv6 bridge 20260522
const CSS_IPV6_INPUT: &str = " div > input#ipv6" ;

// текс Js Injection для выбора Bridges
const JS_INJECTION: &str = r##"
const div = document.querySelector("#bridgelines");
const textNodes = Array.from(div.childNodes)
        .filter(node => node.nodeType === Node.TEXT_NODE)
        .map(node => node.textContent.trim())
        .filter(text => text.length > 0);
textNodes
"## ;

// timeout навигации при клике на кнопку
const TIMEOUT_BUTTON_NAVIGATION: u64 = 120 ; // timeout ожидания помле нажания на кнопку (secs)   // 60 ;

// время ожидания запуска браузера
const TIMEOUT_LANCHING_BROWSER: u64 = 120 ;

// название  раздела
const NAME_PART: &str = "[TOR_BRIDGE]" ;

/// Типы TOR Bridge
#[derive(
    //Debug, 
    //PartialEq,
    //Deserialize,
    Serialize,
    AsRefStr    // Преобразует варианты перечислений в строку &'a
)]
pub enum TypeBridge {
    Vanilla,
    Obfs4,
    Webtunnel,
}   

/// получить LaunchOptions
pub fn get_options() ->LaunchOptions {
    LaunchOptions {
            headless:   true,  //true, // false,
            args:       vec![
                format!(
                    r#"--proxy-server={PROXY_URL}"#
                ).into(),
            ],
            startup_timeout:    Duration::from_secs(TIMEOUT_LANCHING_BROWSER),
            ..LaunchOptions::default()
    }    
}

/// получение мостов от TOR
pub async fn get_britges(
                my_brow:        &mut my_browser::MyBrowser,
                type_bridge:    &TypeBridge,    // тип Bridg для получения
                need_ipv6:      bool,        // признак необходимости ipv6
             ) ->Result<Vec<String>, Box<dyn Error>> {

    // загрузка страницы
    /*
    let page_nat_res = match my_brow.load_url(URL_BRIDGE).await {
        Ok(p_n) => {
            info!("{NAME_PART} успешно загружена страница: {URL_BRIDGE}") ;
            p_n
        },
        Err(err) => {
            let text_mess = format!("url: {URL_BRIDGE}, ошибка при загрузке: {err}") ;
            error!("{URL_BRIDGE} {text_mess}") ;
            return Err(text_mess.into()) ;
        },
    };
     */
    match my_brow.load_url(URL_BRIDGE).await {
        Ok(_) => {
            info!("{NAME_PART} успешно загружена страница: {URL_BRIDGE}") ;
        },
        Err(err) => {
            let text_mess = format!("url: {URL_BRIDGE}, ошибка при загрузке: {err}") ;
            error!("{NAME_PART} {text_mess}") ;
            return Err(text_mess.into()) ;
        },
    }
   
    // Передача в locator css selector для поиска копки получения Bridges
    // и типов требуемых Bridge
    let (button, transport, ipv6_input) = match &my_brow.tab {
        Some(p) => {
            (
                p.locator(CSS_BUTTON),
                p.locator(CSS_TYPE_BRIDGE),
                p.locator(CSS_IPV6_INPUT),  // 20260522
            )
        },
        None => {
            let text_error = "Tab не инициализирован" ;
            error!("{NAME_PART} {text_error}") ;
            return Err(text_error.into());
        },
    } ;

    // поиск копки получения Bridges
    match button.exists().await {
        Ok(res) => {
            if ! res {
                let text_mess = "Не найдена кнопка получения новых Bridges" ;
                error!("{NAME_PART} {text_mess}") ;
                return Err(text_mess.into());
            }
        },
        Err(err) => {
            let text_mess = format!("Ошибка поиска кнопки получения новых Bridges: {err}") ;
            error!("{NAME_PART} {text_mess}") ;
            return Err(text_mess.into());
        },
    }

    // поиск элемента типа Bridge
    match transport.exists().await {
        Ok(res) => {
            if ! res {
                let text_mess = "Не найден элемент типа Bridge" ;
                error!("{NAME_PART} {text_mess}") ;
                return Err(text_mess.into());                
            }
        },
        Err(err) => {
            let text_mess = format!("Ошибка поиска элемента типа Bridge: {err}") ;
            error!("{NAME_PART} {text_mess}") ;
            return Err(text_mess.into());
        },
    }

    // поиск элемента типа ipv6 input 20260522
    match ipv6_input.exists().await {
        Ok(res) => {
            if ! res {
                let text_mess = "Не найден элемент типа IPv6 input" ;
                error!("{NAME_PART} {text_mess}") ;
                return Err(text_mess.into());                
            }
        },
        Err(err) => {
            let text_mess = format!("Ошибка поиска элемента типа IPv6 input: {err}") ;
            error!("{NAME_PART} {text_mess}") ;
            return Err(text_mess.into());
        },
    }

    //println!("type_bridge: {}", type_bridge.as_ref().to_lowercase()) ;

    // установка типа Bridge
    if let Err(err) = transport.select(type_bridge.as_ref().to_lowercase()).await {
        let text_mess = format!("Ошибка установки типа Bridge: {}, error: {err}",type_bridge.as_ref()) ;
        error!("{NAME_PART} {text_mess}") ;
        return Err(text_mess.into());
    }

    /*
    // запуск навигацилнной задачи по отслеживаниюзагрузки новой страницы после клика
    let navigation_task = tokio::spawn({
            let page = my_brow.tab.clone();
            async move {
                page.wait_for_navigation(LoadState::Load, Duration::from_secs(10))
                    .await
            }
    });
     */

    
    let p_ref= match &my_brow.tab {
        Some(p) => {
            p
        },
        None => {
            let text_error = "Tab не инициализирован" ;
            error!("{NAME_PART} {text_error}") ;
            return Err(text_error.into());
        },
    } ;

    // запуск навигацилнной задачи по отслеживаниюзагрузки новой страницы после клика
    let navigation_task = tokio::spawn({
            let page = p_ref.clone();
            async move {
                page.wait_for_navigation(
                        LoadState::Load,
                        Duration::from_secs(
                                            // 10
                                            TIMEOUT_BUTTON_NAVIGATION  // 20260515
                                        )
                    )
                    .await
        }
    });    

    if need_ipv6 {  // 20260522 необходим IPv6 адрес bridge
        // клик по IPv6 input 20260522
        if let Err(err) = ipv6_input.click().await {    // ошибка клика по IPv6 input
            navigation_task.abort();    // абортирование tokio задачи
            let text_error = format!("Url: {URL_BRIDGE}, ошибка клика по IPv6 input: {err}") ;
            error!("{NAME_PART} {text_error}") ;
            return Err(text_error.into());
        }
    }

    // клик по кнопке получения Briges
    if let Err(err) = button.click().await {    // ошибка клика по кнопке
        navigation_task.abort();    // абортирование tokio задачи
        let text_error = format!("Url: {URL_BRIDGE}, ошибка клика по кнопке: {err}") ;
        error!("{NAME_PART} {text_error}") ;
        return Err(text_error.into());
    }

    // получение url после навигации
    let url_now = match navigation_task.await {
        Ok(res_url) => {
            match res_url {
                Ok(url) => url,
                Err(err) => {
                    let text_mess = format!("Ошибка навигационной задачи: {err}") ;
                    error!("{NAME_PART} {text_mess}") ;
                    return Err(text_mess.into());
                },
            }
        },
        Err(err) => {
            let text_mess = format!("Ошибка tokio навигационной задачи: {err}") ;
            error!("{NAME_PART} {text_mess}") ;
            return Err(text_mess.into());
        },
    } ;

    info!("{NAME_PART} Успешная навигация на url: {url_now}") ;

    // получение контента с Bridges
    let val = match p_ref.evaluate(JS_INJECTION).await {
        Ok(v) => v,
        Err(err) => {
            let text_mess = format!("Ошибка при выполнении JsInjection: {err}") ;
            error!("{NAME_PART} {text_mess}") ;
            return Err(text_mess.into());
        },
    } ;

    //let mut list_bridges = vec![] ;

    // анализ полученного значения
    match val.as_array() {
        Some(x) => {
            Ok(
                x
                    .iter()
                    .filter_map(|s|
                        s
                            .as_str()
                            .map(String::from)
                    )
                    .collect::<Vec<String>>()
            )
        },
        None => {
            let text_mess = format!("URL: {url_now}, вернулся не массив а: {}", val.to_string()) ;
            error!("{NAME_PART} {text_mess}") ;
            Err(text_mess.into())
        },
    }
}
