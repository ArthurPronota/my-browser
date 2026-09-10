/*

    moat_data.rs


    // -------------------------------

url: https://bridges.torproject.org/meek/moat/fetch

                Ошибка:
Response {
    url: Url {
        scheme: "https",
        cannot_be_a_base: false,
        username: "",
        password: None,
        host: Some(
            Domain(
                "bridges.torproject.org",
            ),
        ),
        port: None,
        path: "/meek/moat/fetch",
        query: None,
        fragment: None,
    },
    status: 400,
    headers: {
        "date": "Mon, 25 May 2026 14:29:28 GMT",
        "server": "Apache",
        "x-content-type-options": "nosniff",
        "x-content-type-options": "nosniff",
        "x-frame-options": "sameorigin",
        "x-xss-protection": "1",
        "referrer-policy": "no-referrer",
        "strict-transport-security": "max-age=15768000; preload",
        "content-security-policy": "default-src 'none'; base-uri 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; font-src 'self';",
        "onion-location": "http://yq5jjvr7drkjrelzhut7kgclfuro65jjlivyzfmxiq2kyv5lickrl4qd.onion/meek/moat/fetch",
        "content-type": "text/plain; charset=utf-8",
        "content-length": "13",
        "via": "1.1 bridges.torproject.org",
        "connection": "close",
    },
}

                Нормальный ответ:
Response {
    url: Url {
        scheme: "https",
        cannot_be_a_base: false,
        username: "",
        password: None,
        host: Some(
            Domain(
                "bridges.torproject.org",
            ),
        ),
        port: None,
        path: "/moat/fetch",
        query: None,
        fragment: None,
    },
    status: 200,
    headers: {
        "date": "Mon, 25 May 2026 14:33:09 GMT",
        "server": "Apache",
        "x-content-type-options": "nosniff",
        "x-frame-options": "sameorigin",
        "x-xss-protection": "1",
        "referrer-policy": "no-referrer",
        "strict-transport-security": "max-age=15768000; preload",
        "content-security-policy": "default-src 'none'; base-uri 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; font-src 'self';",
        "onion-location": "http://yq5jjvr7drkjrelzhut7kgclfuro65jjlivyzfmxiq2kyv5lickrl4qd.onion/moat/fetch",  <- можно запросить его
        "content-type": "application/json",
        "via": "1.1 bridges.torproject.org",
        "transfer-encoding": "chunked",
    },
}


    Запрос значения по ключу "onion-location" если нужно

    Тело ответ:
{
  "data": [
    {
      "id": "1",
      "type": "moat-challenge",
      "version": "0.1.0",
      "transport": ["obfs4"],
      "image": "/9j/4AAQSkZJRgABAQEAlgCWAAD/...Z//Z",
      "challenge":"obfs4"
    }
  ]
}


        POST тело запрос для получения с решением капчи (solution):
{
  "data": [{
    "id": 2,
    "type": "moat-solution",
    "version": "0.1.0",
    "transport": "obfs4",
    "challenge": "<тот же challenge из fetch>",
    "solution": "<текст с картинки>",
    "qrcode": "false"
  }]
}

*/

use reqwest ;

use serde_json::{self, json} ;

use serde::{
        Deserialize, 
        Serialize, 
        //de::Error
    } ;

use std::{error::Error, time::Duration} ;

use log::{error, info};

use crate::{telegraph, tor_bridge} ;

/// Внутренние даннын
#[derive(Deserialize, Debug)]
struct DataIntResp {
    id:         String,
    r#type:     String,
    version:    String,
    transport:  Vec<String>,
    image:      String,
    challenge:  String,
}

// Данные ответа на запрос bridges
#[derive(Deserialize, Debug)]
struct DataResp {
    data:   Vec<DataIntResp>
}


/// Внутренние данные с TOR Bridges
#[derive(Deserialize, Debug)]
struct BridgesDataInt {
    id: String,
    r#type: String,
    version: String,
    bridges: Vec<String>,
    qrcode:  Option<String>,
}

/// Полученные TOR Bridges
#[derive(Deserialize, Debug)]
struct BridgesData {
    data: Vec<BridgesDataInt>
}

/// Решение капчи с капча сервера
#[derive(Deserialize, Debug)]
struct CaptchaSolution {
    status:     String,
    solution:   String,
}

const BASE_URL: &str = "https://bridges.torproject.org" ;   // базовый url

const PATH_FETCH: &str = "/moat/fetch" ; // "/meek/moat/fetch" ;   // путь запроса bridges

const PATH_CHECK: &str = "/moat/check" ;    // путь для проверки и получения бриджес 20260525

const CONTENT_TYPE: &str = "Content-Type" ; // ключ Content-Type

const CONTENT_TYPE_VALUE: &str = "application/vnd.api+json" ;   // тип контента

const CAPTCHA_URL: &str = "http://127.0.0.1:8000/solve" ;    // url для решения captca

const CAPTCHA_SHUTDOWN_URL: &str = "http://127.0.0.1:8000/shutdown" ; // url для остановки скрипта распознавания капчи 20260529

const CAPTCH_DEFAULT: &str = "freedom" ;    // текст капчи по умолчанию 20260526

use std::process::Command ; // конструктор процессов 20260528

// название  раздела
const NAME_PART: &str = "[MOAT]" ;

// получение мостов по протоколу Moat
pub async fn get_bridges(
                type_bridge: &tor_bridge::TypeBridge // нужный тип мостов
             ) ->Result<Vec<String>, Box<dyn Error>>{

    /*
    //let client = reqwest::Client::new() ;   // создать нового клиенте request
    let client = reqwest::Client::builder() ;
     */

    let request_body = json!({  // тело запроса
        "data": [{
            "version": "0.1.0",
            "type": "client-transports",
            "supported": [
                //"obfs4"
                type_bridge.as_ref().to_lowercase() // 20260526
            ]
        }]
    });

    // создать proxy
    let proxy = match reqwest::Proxy::all(
                            tor_bridge::PROXY_URL
                            //"socks5://127.0.0.1:9150"
                        ) {
            Ok(pr) => pr,
            Err(err) => {
                let text_mess = format!("{NAME_PART} ошибка создания proxy: {err}") ;
                error!("{text_mess}") ;
                return Err(text_mess.into()) ;
            },
    } ;

    // создание http клиента
    let client = match reqwest::Client::builder()
                                .proxy(proxy)
                                .timeout(Duration::from_secs(30))
                                .build() {
        Ok(cl) => cl,
        Err(err) => {
            let text_mess = format!("{NAME_PART} ошибка создания http client: {err}") ;
            error!("{text_mess}") ;
            return Err(text_mess.into()) ;
        },
    } ;

    // url для POST запроса
    let url = format!("{BASE_URL}{PATH_FETCH}") ;

    //println!("url: {url}") ;

    // формируем и оптравляем POST запрос для получения bridges
    let res = match client
            .post(&url)
            .header(CONTENT_TYPE, CONTENT_TYPE_VALUE)
            .json(&request_body)
            .send()
            .await {
        Ok(v) => v,
        Err(err) => {
            let text_mess = format!("{NAME_PART} url: {url}, ошибка при выполнении POST запроса: {err}") ;
            error!("{text_mess}") ;
            return Err(text_mess.into()) ;
        }
    } ;

    //println!("Ответ: {res:#?}") ;

    // получить полный текст ответа
    let content_body = match res.text().await {
       Ok(c) => c,
       Err(err) => {
            let text_mess = format!("{NAME_PART} ошибка получения контента url: {url}: {err}") ;
            error!("{text_mess}") ;
            return Err(text_mess.into()) ;
       }
    } ;

    //println!("content_body: {content_body}") ;

    let cont_resp = match serde_json::from_str::<DataResp>(&content_body) {
        Ok(v) => v,
        Err(err) => {
            let text_mess = format!("{NAME_PART} ошибка десириализации контента url: {url}: {err}") ;
            error!("{text_mess}") ;
            return Err(text_mess.into()) ;
        },
    } ;

    //println!("{cont_resp:#?}") ;

    let need_index = 0 ;    // необходимый индекс

    // проверка индекса
    if let None = cont_resp.data.get(need_index) {
        let text_mess = format!("{NAME_PART} нет данных по индексу: cont_resp.data[{need_index}]") ;
        return Err(text_mess.into())
    }

    // ************ Распознавание капчм **************
    // значение капчи по умолчанию
    let mut solution_captcha = CAPTCH_DEFAULT.to_string() ;

    let client_captch = reqwest::Client::new(); // создание клиентя для опознавания капчи

    // Зарос капчи
    let response_captcha = match client_captch
            .post(CAPTCHA_URL)
            .timeout(Duration::from_secs(5))
            .json(
                &json!({
                    "image_base64": &cont_resp.data[need_index].image
                })
            )
            .send()
            .await {
        Ok(res) => res,
        Err(err) => {
            let text_mess = format!("{NAME_PART} Ошибка запроса капчи: {err}") ;
            return Err(text_mess.into())
        },
    } ;

    if response_captcha.status().is_success() { // успешный ответ от капча сервера
        // ожидание получения полного ответа 
        match response_captcha.text().await {
            Ok(text_resp) => {
                // получение текста распознанной картинки
                solution_captcha = match serde_json::from_str::<CaptchaSolution>(&text_resp) {
                    Ok(s) => {
                        if s.solution.is_empty() {
                            error!("{NAME_PART} Текст капчи пустой, принимаем значение по умолчанию") ;
                            CAPTCH_DEFAULT.to_string()
                        } else {
                            info!("{NAME_PART} Распознанный текст с капчи: {}", s.solution) ;
                            s.solution
                        }
                    },
                    Err(err) => {
                        let text_mess = format!("{NAME_PART} Ошибка десириализации решения капчи: {err}") ;
                        return Err(text_mess.into())
                    },
                } ;
            },
            Err(err) => {
                let text_mess = format!("{NAME_PART} Ошибка получения полного ответа расознавания капчи: {err}") ;
                return Err(text_mess.into())
            },
        }
        //println!("response_captcha: {response_captcha:?}") ;
    } else {    // ошибочный ответ от капча сервера
        match response_captcha.status().as_u16() {
            422 | 400 => {
                error!("{NAME_PART} Ошибка распознования капчи, принимаем значение по умолчанию") ;
                solution_captcha = CAPTCH_DEFAULT.to_string() ;
            },
            http_code => {  // прочие коды возврата
                let text_mess = format!("{NAME_PART} Ошибочный код возврата от капчи сервера: {http_code}") ;
                return Err(text_mess.into())
            },
        }
    }

    // формирование тела POST json решения для ответа
    let solution_body = json!({
        "data": [{
            "id": 2,
            "type": cont_resp.data[need_index].r#type, // "moat-solution",
            "version": cont_resp.data[need_index].version,   // "0.1.0",
            "transport": cont_resp.data[need_index].transport,  // ["obfs4"],
            "challenge": cont_resp.data[need_index].challenge,  // "obfs4"
            "solution":  &solution_captcha,  // "freedom",    // "<текст с картинки>",
            "qrcode": "false"    // нужен ли QR код
        }]
    }) ;

    // url для POST запроса
    let url = format!("{BASE_URL}{PATH_CHECK}") ;

    // формируем и оптравляем POST запрос с решением для получения bridges
    let res = match client
            .post(&url)
            .header(CONTENT_TYPE, CONTENT_TYPE_VALUE)
            .json(&solution_body)
            .send()
            .await {
        Ok(v) => v,
        Err(err) => {
            let text_mess = format!("{NAME_PART} url: {url}, ошибка при выполнении POST запроса: {err}") ;
            error!("{text_mess}") ;
            return Err(text_mess.into()) ;
        }
    } ;

    if ! res.status().is_success() {
        let text_mess = format!("{NAME_PART} url: {url}, ошибка при выполнении POST запроса, code: {}", res.status().as_u16()) ;
        error!("{text_mess}") ;
        return Err(text_mess.into()) ;
    }

    //println!("Ответ: {res:#?}") ;

    // получить полный текст ответа
    let content_body = match res.text().await {
       Ok(c) => c,
       Err(err) => {
            let text_mess = format!("{NAME_PART} ошибка получения контента url: {url}: {err}") ;
            error!("{text_mess}") ;
            return Err(text_mess.into()) ;
       }
    } ;

    //println!("content_body: {content_body}") ;

    let bridges_data = match serde_json::from_str::<BridgesData>(&content_body) {
        Ok(bs) => bs,
        Err(err) => {
            let text_mess = format!("{NAME_PART} ошибка десириализации контента url: {url}: {err}") ;
            error!("{text_mess}") ;
            return Err(text_mess.into()) ;
        },
    } ;

    let need_index = 0 ;    // необходимый индекс

    match bridges_data.data.get(need_index) {
        Some(bs) => {   // индекс найден
            if bs.bridges.is_empty() {  // нет TOR Bridges
                let text_mess = format!("{NAME_PART} bridges_data.data.bridges пуст") ;
                Err(text_mess.into())
            } else {    // возврат TOR Bridges
                Ok(bs.bridges.clone())
            }
        },
        None => {   // индекса нет
            let text_mess = format!("{NAME_PART} нет данных по индексу: bridges_data.data[{need_index}]") ;
            Err(text_mess.into())
        }
    }
}

/// Остановка сервера распознавания капчи 20260528
pub async fn shutdown_server() ->Result<(), Box<dyn Error>>{
    let client_captch = reqwest::Client::new(); // создание клиентя для опознавания капчи

    // Зарос остановки сервера распознавания капчи
    match client_captch
            .post(CAPTCHA_SHUTDOWN_URL)
            .timeout(Duration::from_secs(5))
            .json(
                &json!({})
            )
            .send()
            .await {
        Ok(_) => Ok(()),
        Err(err) => {
            let text_mess = format!("{NAME_PART} Ошибка запроса остановки капчи: {err}") ;
            error!("{text_mess}") ;
            return Err(text_mess.into())
        },
    }
}

/// Запуск сервера распознавания капчи 20260528
pub fn run_server() ->Result<(), Box<dyn  Error>> {

    match Command::new(r"captcha_solution/env/Scripts/python")
            .arg(r"captcha_solution/server.py")
            .spawn() {
        Ok(v) => Ok(()),
        Err(err) => {
            let text_mess = format!("{NAME_PART} Ошибка запуска сервера распознавания капчи: {err}") ;
            error!("{text_mess}") ;
            return Err(text_mess.into());
        }
    }
}
