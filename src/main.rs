
use std::collections::HashMap;

use std::env::temp_dir;

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use std::{
        collections::linked_list, 
        os::windows::process, 
        time::Duration
    };

use playhard::Browser;
use tokio::signal;  // асинхронная обработка сигналов для Tokio
use tokio::sync::RwLock;

use std::sync::{
        Arc, 
        //RwLock
    };

mod my_browser; // модуль браузера

mod tor_bridge; // модуль получения TOR Bridge

//mod pastebin ;  // модуль сохранения результатов в pastebin.com

mod telegraph;  // модуль работы с telegra.ph

mod moat_data;  // moad для получение TOR Bridges

use env_logger ;    // простой логер

use log::{error, info} ;
use serde::Deserialize;

/*
use arrayvec::ArrayVec ;    // вектор, основанный на массиве фиксированного размера. 20260519
 */

/// Структура данных telegraph параметров 20260517
#[derive(Deserialize)]
struct MyTelegraph {
    short_name:     String,
    author_name:    String,
    access_token:   String,
}

/// Структура для конфигурационного файла
#[derive(
    Deserialize,
    //Debug
)]
struct Config {
    username:   String,
    password:   String,
    telegraph:  MyTelegraph,    // 20260517
}

// timeout между запусками получения данных
const TIME_OUT_BETWEEN_RUN: u64 = 180 ;  // sec

// максимальное количество попыток взятия страницы
const MAX_TRY_GET_PAGES: u16 = 100 ;

// время между попытками получить bridges
const TIME_BETWEEN_PAGES: u64 = 5 ;     // secs

// конфигурационный файд
const CONFIG_FILE: &str = "config.toml" ;

const ACTUAL_CAPACITY_VEC: usize = 100 ; // 20260522 колтчество мостов для одного типа моста :   17 ; // 1; //20 ;   // актуальня вместимость Bridges

const TIMEOUT_CLEAR_COOKIE: u64 = 30 ; // timeout операции очистки cookie в браузере 20260528

/// все используемые типы мостов
const ALL_TYPES_BRIDGES: &[tor_bridge::TypeBridge] = &[
                            tor_bridge::TypeBridge::Obfs4,
                            /* 20260522 Пока не используем Webtunnel
                            tor_bridge::TypeBridge::Webtunnel
                             */
                        ] ;

const NAME_PART: &str = "[MAIN]" ;  // название части

// Точка входа
#[tokio::main]
pub async fn main() -> ! {  // функция никогда не возвращает управление (never type) 

    env_logger::init(); // инициализация глобального логера

    // чтение содержимого конфигурационного файла
    let f_cont = match std::fs::read_to_string(CONFIG_FILE) {
        Ok(f_c) => f_c,
        Err(err) => {
            error!("{NAME_PART} fs::read_to_string error: {err}") ;
            std::process::exit(-1) ;
        }
    } ;

    // сощданиеконфигурационной структуры
    let conf = match toml::from_str::<Config>(&f_cont) {
        Ok(c) => c,
        Err(err) => {
            error!("{NAME_PART} toml::from_str error: {err}") ;
            std::process::exit(-1) ;         
        },
    } ;

    info!("{NAME_PART} конфигурация считана") ;

    //println!("conf: {conf:?}") ;

    // список успешно отправленных bridges
    //let mut success_sended_bridges: Vec<String> = vec![] ;

    /*
    // начальная инициация векторя для obfs4 Bridges 20260519
    let mut vec_obfs4: ArrayVec<String, ACTUAL_CAPACITY_VEC> = ArrayVec::new() ;

    // начальная инициация вектор для webtunnel Bridges 20260519
    let mut vec_webtunnel: ArrayVec<String, ACTUAL_CAPACITY_VEC> = ArrayVec::new() ;
     */
    
    // признак начальной загрузки страницы 20260519
    // let mut first_page_loaded = false ;

    // призенак начальной загрузки страиц 20250522
    // let mut first_loading = false ;

    /* 20260527
    // начальная инициация браузера 20260527
    let mut browser = my_browser::MyBrowser::new() ;    

    // установка опцмй браузера
    browser.set_options(&tor_bridge::get_options());
     */
    /*
    if let Err(err) = ctrlc::set_handler(|| {
        &browser.shutdown_browser().await ;

        }) 
    {
        info!("{NAME_PART} Ошибка установкии обработчика CntrlC: {err}") ;
        std::process::exit(-1) ;
    }


    info!("{NAME_PART} Запуск обработчик CntrlC") ;
    */

    /*
    let brw = Arc::new(RwLock::new(browser)) ;
    tokio::spawn({
        let brw = brw.clone() ;
        async move {
            match signal::ctrl_c().await {
                Ok(_) => {
                    info!("{NAME_PART} Закрытие браузера при получении CntrlC") ;
                    // закрытие браузера
                    let _ = brw.write().await.shutdown_browser().await ;
                    info!("{NAME_PART} Браузер закрыт при получении CntrlC !!!!!!!!!!") ;
                    std::process::exit(-1) ;
                },
                Err(err) => {
                    error!("{NAME_PART} Ошибка при ожидании получения сигнала CntrlC: {err}") ;
                },
            }
        }
    }) ;

    info!("{NAME_PART} Запущен обработчик CntrlC") ;
     */

    // Запуск сервера распознавания капчи 20260528
    if let Err(_) = moat_data::run_server() {
        std::process::exit(0) ;
    }
    
    // нажата ли CntrlC
    let is_cntl_c = Arc::new(AtomicBool::new(false)) ;

    // это начальный timeout
    let is_start_timeout = Arc::new(AtomicBool::new(false)) ;

    // обработка сигнала CntrlC
    tokio::spawn({
        let is_cntl_c = is_cntl_c.clone() ;
        let is_start_timeout = is_start_timeout.clone() ;
        async move {
            match signal::ctrl_c().await {
                Ok(_) => {
                    info!("{NAME_PART} Получен сигнал CntrlC") ;
                    let _ = moat_data::shutdown_server().await ;    // 20260528 Остановка сервера распознавания капчи
                    is_cntl_c.store(true, Ordering::Release);
                    //let _ = moat_data::shutdown_server().await ;    // 20260528 Остановка сервера распознавания капчи
                    if is_start_timeout.load(Ordering::Acquire) {   // это длинное начальное ожидание 20260527
                        std::process::exit(0) ;
                    }
                },
                Err(err) => {
                    error!("{NAME_PART} Ошибка при ожидании получения сигнала CntrlC: {err}") ;
                },
            }
        }
    }) ;

    let mut ind = 0u64 ;
    // вечный цикл
    loop {

        is_start_timeout.store(false, Ordering::Release);      // 20260527

        if ind > 0 {
            if ! is_cntl_c.load(Ordering::Acquire) {    // CntrlC не нажат 20270527
                is_start_timeout.store(true, Ordering::Release);       // 20260527
                tokio::time::sleep(Duration::from_secs(TIME_OUT_BETWEEN_RUN)).await ;
            }
        }

        is_start_timeout.store(false, Ordering::Release);   // 20260527

        // начальная инициация браузера
        //* 20260527
        let mut browser = my_browser::MyBrowser::new() ;
        //*/

        //* 20260528 !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
        //browser = my_browser::MyBrowser::new() ;    // 20260527

        // установка опцмй браузера
        browser.set_options(&tor_bridge::get_options());
        // */

        /* 20260527
        let mut browser = brw.write().await ;

        browser.new2();   // 20260527
        browser.set_options(&tor_bridge::get_options()); // 20260527
         */

        //tokio::time::sleep(Duration::from_millis(1)).await ;    // 20260527

        if browser.need_run() {    // необходимость инициализации
            if let Err(_) = browser.launch_browser().await {
                continue ;
            }
        }

        // проверка необъодимости остановки браузера
        check_need_stop_browser(&is_cntl_c, &mut browser).await ;

        //tokio::time::sleep(Duration::from_millis(1)).await ;    // 20260527

        /* 20260527 Подвисание
        // очистка cookie браузера 20260519
        if let Err(_) = browser.clear_all_cooie().await {
            let _ = browser.shutdown_browser().await ;
            continue;
        }
         */
        // очистка cookie в браузере с контролем подвисания 20260528
        match tokio::time::timeout(
                    Duration::from_secs(TIMEOUT_CLEAR_COOKIE),
                    async {
                        browser.clear_all_cooie().await
                    }
                ).await {
            Ok(Ok(_)) => {},    // cookie очищены
            Ok(Err(_)) => {   // ошибка очистки cookie
                let _ = browser.shutdown_browser().await ;
                continue;
            },
            Err(err) => {   // Ошибка timeout
                error!("{NAME_PART} Ошибка timeout по очистке cookie в браузере: {err}") ;
                let _ = browser.shutdown_browser().await ;
                continue;                
            },
        }
        
        // проверка необъодимости остановки браузера
        check_need_stop_browser(&is_cntl_c, &mut browser).await ;

        //tokio::time::sleep(Duration::from_millis(1)).await ;    // 20260527

        if browser.need_new_tab() { // необходимо открыть new Tab
            if let Err(_) = browser.new_tab().await {
                let _ = browser.shutdown_browser().await ;
                continue ;                
            }
        }

        // проверка необъодимости остановки браузера
        check_need_stop_browser(&is_cntl_c, &mut browser).await ;

        //tokio::time::sleep(Duration::from_millis(1)).await ;    // 20260527
                
        // выходные bridges 20260523
        //let mut out_bridges: HashMap<String, u64> = HashMap::new() ;

        // список страниц с tekegra.ph 20260502
        //let mut telegraph_list_path: Vec<String> = vec![] ;

        //* * * * * * * * * * * *
        // список всех Bridges c сайта TOR
        let mut list_bridges_from_tor: Vec<String> = vec![] ;

        let mut is_normal = false ;    // признак нормально получения bridges

        // получение всех необъодимых видов bridges с сайта TOR
        for type_br in ALL_TYPES_BRIDGES    // 20260522
                        /* 20260522
                        [
                            tor_bridge::TypeBridge::Obfs4,
                            /* 20260522 Пока не используем Webtunnel
                            tor_bridge::TypeBridge::Webtunnel
                             */
                        ] 
                        */
                        {

            is_normal = false ; // 20260522

            // перебор использования IPv6   20260522
            for need_ipv6 in [false, true] {

                is_normal = false ;
                // Заогрузка всех Bridges указанного типа
                for i in 0..MAX_TRY_GET_PAGES {

                    // проверка необъодимости остановки браузера
                    check_need_stop_browser(&is_cntl_c, &mut browser).await ;

                    if i > 0 {
                        tokio::time::sleep(Duration::from_secs(TIME_BETWEEN_PAGES)).await ;
                    }
                    // получить bridges с сайта tor
                    match tor_bridge::get_britges(
                                    &mut browser, 
                                    &type_br,
                                    need_ipv6   // 20260522
                                ).await {
                        Ok(v) => {  // данные получены
                            is_normal = true ;
                            list_bridges_from_tor.extend(v);
                            break ;
                        },
                        Err(_) => { // ошибка получения данных
                            is_normal = false ;
                            continue;
                        }
                    } ;
                }

                if ! is_normal {
                    error!("{NAME_PART} Ошибка получения Bridges") ;
                    break ;
                }
            }

            if ! is_normal {
                error!("{NAME_PART} Ошибка получения Bridges") ;
                break ;
            }
        }

        //println!("is_normal: {is_normal}") ;

        if ! is_normal {    // аварийное завершение сбора данных
            let _ = browser.shutdown_browser().await ;
            continue;
        }

        if list_bridges_from_tor.len() == 0 {
            error!("Список Bridges пуст") ;
            let _ = browser.shutdown_browser().await ;
            continue;            
        }

        /* 20260519
        println!("list_all_bridges: {list_all_bridges:#?}") ;
         */

        //  Получение Bridges с Moat 20260526 ***************
        let mut is_normal = false ;    // признак нормально получения bridges

        for type_br in ALL_TYPES_BRIDGES {
            is_normal = false ; // 20260522

            // Заогрузка всех Bridges указанного типа
            for i in 0..MAX_TRY_GET_PAGES {

                // проверка необъодимости остановки браузера
                check_need_stop_browser(&is_cntl_c, &mut browser).await ;

                if i > 0 {
                    tokio::time::sleep(Duration::from_secs(TIME_BETWEEN_PAGES)).await ;
                }

                match moat_data::get_bridges(&type_br).await {
                    Ok(br) => { // данные получены
                        is_normal = true ;
                        list_bridges_from_tor.extend(br);
                        break ;
                    },
                    Err(_) => {   // ошибка получения данных
                        is_normal = false ;
                        continue;
                    },
                }
            }

            if ! is_normal {
                error!("{NAME_PART} Ошибка получения Bridges с Moat") ;
                break ;
            }
        }

        if ! is_normal {    // аварийное завершение сбора данных
            let _ = browser.shutdown_browser().await ;
            continue;
        }

        /* 20260526
        println!("list_bridges_from_tor: {list_bridges_from_tor:#?}") ;
         */

        // ************ получить time stamp now ************
        let time_stamp_now = match SystemTime::now()
                                            .duration_since(
                                               UNIX_EPOCH 
                                            ) {
            Ok(dur_ts) => {
                dur_ts.as_secs()
            },
            Err(err) => {
                error!("{NAME_PART} duration_since(UNIX_EPOCH) error: {err}") ;
                let _ = browser.shutdown_browser().await ;
                continue;
            },
        } ;

        let mut out_bridges = HashMap::new() ;  // начальная установка выходных bridges 20260523

        // анализ полученных bridges 20260522
        for brg in &list_bridges_from_tor {
            if brg[0..=tor_bridge::TypeBridge::Obfs4.as_ref().len()] == tor_bridge::TypeBridge::Obfs4.as_ref().to_lowercase() + " " {
                //println!("Это Obfs4: {brg}") ;
                out_bridges.insert(brg.to_owned(), time_stamp_now) ; // 20260503

            } else if brg[0..=tor_bridge::TypeBridge::Webtunnel.as_ref().len()] == tor_bridge::TypeBridge::Webtunnel.as_ref().to_lowercase() + " " {
                //println!("Это Webtunnel: {brg}") ;
                out_bridges.insert(brg.to_owned(), time_stamp_now) ; // 20260503
            } else if ! telegraph::brg_is_link(brg) {
                error!("Не обрабатываемый тип bridge: {brg}") ;
            }
        }

        /*
        list_bridges_from_tor = vec![] ;    // 20260522 обнуляем списка полученных от TOR мостов

        // загрузка всех obfs4 bridges
        for s_tmp in &vec_obfs4 {
            list_bridges_from_tor.push(s_tmp.to_string());
        }

        // загрузка всех webtunnel bridges
        for s_tmp in &vec_webtunnel {
            list_bridges_from_tor.push(s_tmp.to_string());
        }
         */

        /* 20260519
        list_all_bridges.sort();
         */

        // ************ Загрузка перечня страниц с telegra.ph ************

        is_normal = false ;

        let mut telegraph_list_path = vec![] ;

        // получение списка страниц с telgra.ph
        for i in 0..MAX_TRY_GET_PAGES {

            // проверка необъодимости остановки браузера
            check_need_stop_browser(&is_cntl_c, &mut browser).await ;

            if i > 0 {
                tokio::time::sleep(Duration::from_secs(TIME_BETWEEN_PAGES)).await ;
            }

            // получение списка страниц
            match telegraph::get_list_pages(&mut browser, &conf.telegraph.access_token).await {
                Ok(ps) => {
                    is_normal = true ;
                    telegraph_list_path = ps ;
                    break ;
                },
                Err(_) => {
                    is_normal = false;
                    continue;
                }
            }
        }

        if ! is_normal {
            error!("{NAME_PART} список страниц с telegra.ph не получен.") ;
            let _ = browser.shutdown_browser().await ;
            continue;
        } else {
            info!("{NAME_PART} список страниц telegra.ph получен") ;
        }

        // ************ загрузка информации о bridge со страниц telegra.ph ************

        is_normal = false ;

        for url in &telegraph_list_path {
            is_normal = false ;

            // получение списка мостов с страницы telgra.ph
            for i in 0..MAX_TRY_GET_PAGES {
                // проверка необъодимости остановки браузера
                check_need_stop_browser(&is_cntl_c, &mut browser).await ;

                if i > 0 {
                    tokio::time::sleep(Duration::from_secs(TIME_BETWEEN_PAGES)).await ;
                }                

                // получение списка мостов
                match telegraph::get_bridges_from_page(
                                &mut browser, 
                                &format!("{}{}", telegraph::BASE_PAGE_TELEGRA, url)
                            ).await {
                    Ok(brgs) => {
                        for brg in &brgs {

                            if brg.is_empty() { // контроль моста
                                continue;
                            }

                            if brg[0..=tor_bridge::TypeBridge::Obfs4.as_ref().len()] == tor_bridge::TypeBridge::Obfs4.as_ref().to_lowercase() + " " 
                               || brg[0..=tor_bridge::TypeBridge::Webtunnel.as_ref().len()] == tor_bridge::TypeBridge::Webtunnel.as_ref().to_lowercase() + " " 
                            {
                                let mut t_s = 0u64 ;    // time stamp текущего bridge
                                let parts = brg.split('|').collect::<Vec<&str>>() ;
                                if parts.len() == 2 {   // присутствует bridge и time_stamp
                                    t_s = match parts[1].parse::<u64>() {
                                        Ok(t) => t,
                                        Err(_) => 0
                                    } ;
                                }
                                if ! out_bridges.contains_key(parts[0]) {   // такого моста нет
                                    out_bridges.insert(parts[0].to_string(), t_s) ;
                                }
                            } else if ! telegraph::brg_is_link(brg) {
                                error!("Не обрабатываемый тип bridge: {brg}") ;
                            }
                        }
                        is_normal = true ;
                        break ;
                    },
                    Err(_) => {
                        is_normal = false;
                        continue;
                    }
                }                    
            }

            if ! is_normal {
                error!("{NAME_PART} Ошибка загрузки мостов с telegra.ph/{url}") ;
                let _ = browser.shutdown_browser().await ;
                continue;
            } else {
                info!("{NAME_PART} Загрузка мостов с telegra.ph url: {url} выполнена") ;
            }
        }

        if ! is_normal {
            error!("{NAME_PART} ошибка загрузки мостов со страниц telegra.ph") ;
            let _ = browser.shutdown_browser().await ;
            continue;
        } else {
            info!("{NAME_PART} Загружены мосты со страниц telegra.ph") ;
        }

        /*
        // преобразуем HashMap в Vec
        let vec_out = out_bridges
                                            .into_iter()
                                            .collect::<Vec<(String, u64)>>() ;

        // сортировка в порядке возрастания u64
        vec_out.sort_by(|a, b| a.1.cmp(&b.1)) ;

        // преобразовать vec_out в Vec<String>
        let vec_out = 
                vec_out
                    .into_iter()
                    .map(|x| x.0)
                    .collect::<Vec<String>>() ;

        // разбить Vec<String> в Vec<Vec<String>>
        let vec_out = 
                vec_out
                    .chunks(17)
                    .map(|chunk| chunk.to_vec())
                    .collect::<Vec<Vec<String>>>() ;
         */

        // создание списка мостов для страниц 20260524
        let bridges_to_page = telegraph::convert_bridges_to_chanks(&out_bridges) ;

        // сформировать список: страница => список мостов на ней
        let list_pg_to_brg = 
                match telegraph::make_list_page_to_bridges(&mut telegraph_list_path, &bridges_to_page) {
            Ok(v) => v,
            Err(_) => {
                let _ = browser.shutdown_browser().await ;
                continue;                
            }
        };

        // сохранение на страницах telegra.ph мостов
        let mut pg_old = "".to_string() ;   // предыдущая страница

        is_normal = false ;

        for pg_to_brg in list_pg_to_brg {
            is_normal = false ;

            // создание/редактирование страниц
            for i in 0..MAX_TRY_GET_PAGES {

                // проверка необъодимости остановки браузера
                check_need_stop_browser(&is_cntl_c, &mut browser).await ;

                if i > 0 {
                    tokio::time::sleep(Duration::from_secs(TIME_BETWEEN_PAGES)).await ;
                }

                match telegraph::update_page(
                            &mut browser,
                            &pg_to_brg.0,
                            &pg_old,
                            &conf.telegraph.access_token,
                            &pg_to_brg.1
                    ).await {
                        Ok(path) => {
                            is_normal = true ;
                            //telegraph_list_path = ps ;
                            //success_sended_bridges = list_all_bridges ;
                            pg_old = path.clone() ;
                            info!("{NAME_PART} Bridges добавлены в path: {path}") ;
                            break ;
                        },
                        Err(_) => {
                            is_normal = false;
                            continue;
                        },
                }
            }

            if ! is_normal {
                error!("{NAME_PART} Ошибка создания/модифицикации страницы telegra.ph/{}", pg_to_brg.0) ;
                let _ = browser.shutdown_browser().await ;
                continue;
            } else {
                info!("{NAME_PART} страница telegra.ph/{} создана/модифицирована", pg_to_brg.0) ;
            }
        }

        if ! is_normal {
            error!("{NAME_PART} ошибка сохранения мостов на страницах telegra.ph") ;
            let _ = browser.shutdown_browser().await ;
            continue;
        } else {
            info!("{NAME_PART} Сохранены мосты на страницах telegra.ph") ;
        }

        // ----------------------------------

        let _ = browser.shutdown_browser().await ;

        ind = ind.wrapping_add(1) ;
    }

}

// проверка необхожимости остановить  браузер
async fn check_need_stop_browser(
            is_cntrl_c: &Arc<AtomicBool>,
            browser:    &mut my_browser::MyBrowser,
         )
{
    if is_cntrl_c.load(Ordering::Acquire) { // получен сигнал CntrlC
        let _ = browser.shutdown_browser().await ;
        info!("{NAME_PART} Браузер остановлен.") ;
        //let _ = moat_data::shutdown_server().await ;    // 20260528 Остановка сервера распознавания капчи
        tokio::time::sleep(Duration::from_secs(TIME_BETWEEN_PAGES)).await ;
        std::process::exit(0) ;
    }
}
