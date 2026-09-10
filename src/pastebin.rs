use std::{error::Error, time::Duration} ;

use log::{error, info} ;

use crate::my_browser ;

// url логирования
const LOGIN_URL: &str = "https://pastebin.com/" ; // "https://pastebin.com/login" ;

// css контрольного поля подтверждения что это человек
const CSS_INPUT_CHECKBOX: &str = r#".cb-c input[type="checkbox"]"# ;

const NAME_PART: &str = "[PASTEBIN]" ;

/// выполнить логирования
pub async fn do_login(
                my_brow: &mut my_browser::MyBrowser,
                username:   &str,
                password:   &str
             ) ->Result<(), Box<dyn Error>> {

    match my_brow.load_url(LOGIN_URL).await {
        Ok(v) => {
            info!("{NAME_PART} успешно загружена страница: {LOGIN_URL}") ;
        },
        Err(err) => {
            let text_mess = format!("url: {LOGIN_URL}, ошибка при загрузке: {err}") ;
            error!("{NAME_PART} {text_mess}") ;
            return Err(text_mess.into()) ;
        },
    }


    for i in 0..100 {
        tokio::time::sleep(Duration::from_secs(10)).await ;

        //*
        // создаём CSS Locator для получаем допуск к input check
        let input_check = match &my_brow.tab {
            Some(p) => {
                
                //let v = p.frames().await.unwrap()[0] ;
                //p.locator(CSS_INPUT_CHECKBOX)

                println!("length: {}", p.frames().await.unwrap().len()) ;

                let v = p
                    .frames()
                    .await
                    .unwrap()[0]
                    .evaluate(r#"
const checkbox = document.querySelector('.cb-c');
checkbox.click()
                    "#)
                    .await ;

                    println!("{:?}", v) ;

                    //.locator(CSS_INPUT_CHECKBOX)
            },
            None => {
                let text_error = "Tab не инициализирован" ;
                error!("{NAME_PART} {text_error}") ;
                return Err(text_error.into());
            },
        } ;
        // */

        /*
        match input_check.exists().await {
            Ok(res) => {
                if res {
                    println!("{}", input_check.text_content().await.unwrap()) ;

                    info!("input check найден !!!!!!!!!!!!!!!!#") ;
                    if let Err(err) = input_check.focus().await {
                        error!("{NAME_PART} cannot focus to item") ;
                    }
                    input_check.click().await ;
                } else {
                    info!("input check не найден") ;
                }
            },
            Err(err) => {
                let text_mess = format!("Ошибка поиска input check: {err}") ;
                error!("{NAME_PART} {text_mess}") ;
                return Err(text_mess.into());                
            }
        }
         */
    }

    Ok(())
}
