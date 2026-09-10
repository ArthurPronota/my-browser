
use std::error::Error;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use serde_json ;

use log::{error, info} ;

use url::form_urlencoded ;

use crate::my_browser ;

// Упрощённое представление сузествующей страницы
#[derive(Deserialize, Debug)]
struct Page {
    //url:    String,
    path:   String,
}

// Упрощёное представление списка существующих страниц
#[derive(Deserialize, Debug)]
struct ResultListPages {
    pages:  Vec<Page>
}

// структура сушествующих страниц
#[derive(Deserialize, Debug)]
struct GetPagesList {
    ok:     bool,
    result: ResultListPages,
}

/// Узел данных
#[derive(Serialize)]
struct NodeData {
    tag:        String,
    children:   Vec<String>,
}

impl NodeData {
    // создание нового Vec<None>
    fn new(
        child:               &Vec<String>,
        link_to_prev_page:   &str,   // ссылка на предыдущуб страницу
    ) ->Vec<Self> {
        let mut v_out = vec![] ;
        for cld in child {
            v_out.push(
                NodeData {
                    tag: "p".to_string(),
                    children: vec![cld.to_string()]
                }
            );
        }

        // добавить ссылку на предыдущую страницу 20260524
        v_out.push(
            NodeData {
                tag: "p".to_string(),
                children: vec![link_to_prev_page.to_string()]
            }
        );

        v_out
    }
}


/// результат создания/редактирования страницы
#[derive(Deserialize)]
struct ResultNewEdut {
    path:   String,
}

#[derive(Deserialize)]
struct AnswerNewEdit {
    ok:     bool,
    result: ResultNewEdut,
}

// базовая страница telegra.ph
pub const BASE_PAGE_TELEGRA: &str = "https://telegra.ph/" ;

// базовая часть запроса списка статей
const GET_PAGE_LIST_BASE: &str = "https://api.telegra.ph/getPageList?access_token=" ;

// базовая часть редактирования страницы
const EDIT_PAGE_URL: &str = "https://api.telegra.ph/editPage/" ;

// базовая часть создания страницы
const CREATE_PAGE_URL: &str = "https://api.telegra.ph/createPage" ;

// title для API
const TITLE_TELEGRAPH: &str = "bg" ;

// css для получения Bridges
//const CSS_GET_BRIDGES: &str = "article > div > p" ;

// js injection для получения всех bridjes на странице
const JS_INHECTION: &str = r#"
    Array.from(document.querySelectorAll("article > div > p"))
    .map(p => p.textContent.trim())
"# ;

// размер страницы (количеств мостов на страницу) 20260523
pub const PAGE_SIZE: usize = 13 ; // 17 ;

// максимальное количество страниц для bridges 20260523
pub const MAX_PAGE_COUNT: usize = 9 ;   // 6

// название  раздела
const NAME_PART: &str = "[TELEGRAPH]" ;

// получить список всех страниц
pub async fn get_list_pages(
                my_brow: &mut my_browser::MyBrowser,
                access_token:   &str,
             ) ->Result<Vec<String>, Box<dyn Error>> {

    if access_token.is_empty() {
        let text_mess = "access_token пуст" ;
        error!("{NAME_PART} {text_mess}") ;
        return Err(text_mess.into());
    }

    let url = format!("{GET_PAGE_LIST_BASE}{access_token}") ;

    // загружаем страницу
    match my_brow.load_url(&url).await {
        Ok(_) => {
            info!("{NAME_PART} успешно загружена страница: {url}") ;
        },
        Err(err) => {
            let text_mess = format!("url: {url}, ошибка при загрузке: {err}") ;
            error!("{NAME_PART} {text_mess}") ;
            return Err(text_mess.into()) ;
        },
    }

    // выделяем весть текст на странице
    let content = match &my_brow.tab {
        Some(p) => {
            match p.text_content("*").await {
                Ok(ct) => ct,
                Err(err) => {
                    let text_mess = format!("{NAME_PART} Ошибка выделения текста на странице: {url}") ;
                    error!("{text_mess}") ;
                    return Err(err.into());
                },
            }
        },
        None => {
            let text_error = "Tab не инициализирован" ;
            error!("{NAME_PART} {text_error}") ;
            return Err(text_error.into());
        },
    } ;

    // создаём стрруктуру из строки
    let get_pg_list = match serde_json::from_str::<GetPagesList>(&content) {
        Ok(r) => r,
        Err(err) => {
            let text_mess = format!("{NAME_PART} ошибка десириализации данных списка страниц: {err}") ;
            error!("{text_mess}") ;
            return Err(text_mess.into());
        }
    } ;

    //println!("{get_pg_list:#?}") ;

    // контроль статуса получения списка страниц
    if ! get_pg_list.ok {
        let text_mess = format!("{NAME_PART} Недопустимый статус получения списка страниц: {content}") ;
        error!("{text_mess}") ;
        return Err(text_mess.into());
    }

    let mut all_pages = vec![] ;

    for page in get_pg_list.result.pages {
        all_pages.push(page.path);
    }

    Ok(all_pages)
}

/// модифицировать первую страницу
pub async fn update_page(
                my_brow:        &mut my_browser::MyBrowser,
                //list_pages:     &Vec<String>, // 20260524
                page:           &str,   // 20260524 текузая страница
                prev_page:      &str,   // 20260524 предыдущая страница
                access_token:   &str,
                bridges:        &Vec<String>
            ) ->Result<String, Box<dyn Error>> {

    if access_token.is_empty() {
        let text_mess = "access_token пуст" ;
        error!("{NAME_PART} {text_mess}") ;
        return Err(text_mess.into());
    }

    if bridges.len() == 0 {
        let text_mess = "bridges пуст" ;
        error!("{NAME_PART} {text_mess}") ;
        return Err(text_mess.into());        
    }

    // формирования аргумента для кюча content
    let content_vec = NodeData::new(bridges, prev_page) ;
    let content_str = match serde_json::to_string(&content_vec) {
        Ok(st) => st,
        Err(err) => {
            let text_mess = format!("") ;
            error!("{NAME_PART} Ошика сериализации content_vec: {err}") ;
            return Err(text_mess.into());
        },
    } ;

    // формирование url
    let url = if page.len() == 0 {    // для создния страницы
        format!(r##"{CREATE_PAGE_URL}?access_token={}&title={}&content={}&return_content=false"##,
            form_urlencoded::byte_serialize(access_token.as_bytes()).collect::<String>(),
            form_urlencoded::byte_serialize(TITLE_TELEGRAPH.as_bytes()).collect::<String>(),
            form_urlencoded::byte_serialize(content_str.as_bytes()).collect::<String>(),
        )
    } else {    // для редактирования страницы
        format!(
            r##"{EDIT_PAGE_URL}{}?access_token={}&title={}&content={}&return_content=false"##, 
            form_urlencoded::byte_serialize(page.as_bytes()).collect::<String>(),
            form_urlencoded::byte_serialize(access_token.as_bytes()).collect::<String>(),
            form_urlencoded::byte_serialize(TITLE_TELEGRAPH.as_bytes()).collect::<String>(),
            form_urlencoded::byte_serialize(content_str.as_bytes()).collect::<String>(),
        )
    } ;

    // загружаем страницу
    match my_brow.load_url(&url).await {
        Ok(_) => {
            info!("{NAME_PART} успешно загружена страница: {}...", url.chars().take(100).collect::<String>()) ;
        },
        Err(err) => {
            let text_mess = format!("{NAME_PART} url: {url}, ошибка при загрузке: {err}") ;
            /* 20260528
            error!("{NAME_PART} {text_mess}") ;
             */
            return Err(text_mess.into()) ;
        },
    }

    // выделяем весть текст на странице
    let content = match &my_brow.tab {
        Some(p) => {
            match p.text_content("*").await {
                Ok(ct) => ct,
                Err(err) => {
                    let text_mess = format!("{NAME_PART} Ошибка выделения текста на странице: {url}") ;
                    error!("{text_mess}") ;
                    return Err(err.into());
                },
            }
        },
        None => {
            let text_error = "Tab не инициализирован" ;
            error!("{NAME_PART} {text_error}") ;
            return Err(text_error.into());
        },
    } ;


    // создаём структуру из строки
    let answ_new_edit = match serde_json::from_str::<AnswerNewEdit>(&content) {
        Ok(r) => r,
        Err(err) => {
            let text_mess = format!("{NAME_PART} ошибка десириализации данных создания/редактирования страницы: {err}") ;
            error!("{text_mess}") ;
            return Err(text_mess.into());
        }
    } ;

    // контроль статуса получения списка страниц
    if ! answ_new_edit.ok {
        let text_mess = format!("{NAME_PART} Недопустимый статус создания/редактирования страницы: {content}") ;
        error!("{text_mess}") ;
        return Err(text_mess.into());
    }

    Ok(answ_new_edit.result.path)
}

// получить все bridges со страницы 20260519
pub async fn get_bridges_from_page(
                    my_brow:   &mut my_browser::MyBrowser,
                    first_url: &str,
                ) ->Result<Vec<String>, Box<dyn Error>> {

    if first_url.is_empty() {
        let text_mess = "first_url is empty" ;
        error!("{NAME_PART} {text_mess}") ;
        return Err(text_mess.into());
    }

    // загружаем страницу
    match my_brow.load_url(&first_url).await {
        Ok(_) => {
            info!("{NAME_PART} успешно загружена страница: {first_url}") ;
        },
        Err(err) => {
            let text_mess = format!("url: {first_url}, ошибка при загрузке: {err}") ;
            error!("{NAME_PART} {text_mess}") ;
            return Err(text_mess.into()) ;
        },
    }

    // выделяем весть текст на странице
    match &my_brow.tab {
        Some(p) => {
            let val = match p.evaluate(JS_INHECTION).await {
                Ok(v) => v,
                Err(err) => {
                    let text_mess = format!("{NAME_PART}, url: {first_url}, Ошибка при выполнении JsInjection: {err}") ;
                    error!("{text_mess}") ;
                    return Err(text_mess.into());
                }
            } ;

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
                            .collect()
                    )
                },
                None => {
                    let text_mess = format!("{NAME_PART} URL: {first_url}, вернулся не массив а: {}", val.to_string()) ;
                    error!("{text_mess}") ;
                    Err(text_mess.into())
                },
            }
        },
        None => {
            let text_error = "Tab не инициализирован" ;
            error!("{NAME_PART} {text_error}") ;
            Err(text_error.into())
        },
    }

}


/// конвертировать bridges в Vec мостов для страницы 20260524
pub fn convert_bridges_to_chanks(bridges: &HashMap<String, u64>) ->Vec<Vec<String>> {
    // преобразовать &HashMap<String, u64> в Vec<&String, &u64>
    let mut v = 
            bridges
                .iter()
                .collect::<Vec<_>>() ;

    // сортировка в порядке возрастания u64
    v.sort_by(|a, b| a.1.cmp(&b.1));

    // преобразование Vec<&String, &u64> в Vec<String>
    let v = 
            v
                .iter()
                .map(|x| x.0.to_string() + "|" + &format!("{}", x.1))
                .collect::<Vec<_>>() ;
    
    // преобразование Vec<String> в Vec<Vec<String>>
    v
        .chunks(PAGE_SIZE)
        .map(|x| x.to_vec())
        .collect()
}

/// создать список соответсвия страница - перечень мостов 20260524
pub fn make_list_page_to_bridges(
            telegraph_list_path: &mut Vec<String>,  // список существующих страниц
            bridges_to_page:     &Vec<Vec<String>>  // список мостов для каждой страницы
        ) ->Result<
                Vec<(
                    String,        // страница
                    Vec<String>    // мосты на странице
                )>,
                Box<dyn Error>
            >
        {

    if bridges_to_page.len() == 0 {
        let text_error = "Список мостов для страницы пуст" ;
        error!("{NAME_PART} {text_error}") ;
        return Err(text_error.into()) ;
    }

    if telegraph_list_path.len() > bridges_to_page.len() {  // существующиз страниц > чем нужно
        for _ in 0..telegraph_list_path.len() - bridges_to_page.len() {
            telegraph_list_path.remove(0) ;
        }
    } else if bridges_to_page.len() > telegraph_list_path.len() {   // существующих страниц менбще чем мостов
        for _ in 0..bridges_to_page.len() - telegraph_list_path.len() {
            telegraph_list_path.insert(0, "".to_string());
        }
    }

    // контроль размеров bridges_to_page & telegraph_list_path
    if bridges_to_page.len() != telegraph_list_path.len() {
        let text_error = "Размеры bridges_to_page и telegraph_list_path раздичны" ;
        error!("{NAME_PART} {text_error}") ;
        return Err(text_error.into()) ;
    }

    let mut out_vec = Vec::new() ;

    for ind in 0..bridges_to_page.len() {
        out_vec.push(
                (
                    telegraph_list_path[ind].clone(), 
                    bridges_to_page[ind].clone()
                )
            );
    }

    if out_vec.len() > MAX_PAGE_COUNT { // количество страницы больше чем нужно 20260525 
        for _ in 0..out_vec.len() - MAX_PAGE_COUNT {
            out_vec.remove(0) ;
        }
    }

    Ok(out_vec)
}

/// Проверка является ли строка моста ссылкой 20260528
pub fn brg_is_link(brg_str: &str) ->bool {

    match &brg_str.chars().take(3).collect::<String>() {
        str_cast if &format!("{}-", TITLE_TELEGRAPH) == str_cast || "br-" == str_cast => true,
        _ => false
    }
}
